import React, { useRef, useEffect, useState, useContext } from 'react';
import { 
  Box, 
  Grid, 
  Card, 
  CardContent, 
  Typography, 
  Button, 
  Slider, 
  Switch, 
  FormControlLabel,
  Paper,
  Chip,
  Alert,
  ButtonGroup,
  Divider,
  LinearProgress
} from '@mui/material';
import * as THREE from 'three';
import { WebSocketContext } from '../App';

const DroneControlDashboard = () => {
  const { wsData, setWsData } = useContext(WebSocketContext);
  const mountRef = useRef(null);
  const sceneRef = useRef(null);
  const rendererRef = useRef(null);
  const cameraRef = useRef(null);
  const droneRef = useRef(null);
  const animationIdRef = useRef(null);
  
  const [targetAltitude, setTargetAltitude] = useState(0);
  const [controlMode, setControlMode] = useState('manual');

  useEffect(() => {
    initThreeJS();
    
    return () => {
      cleanup();
    };
  }, []);

  useEffect(() => {
    updateDronePosition(wsData.droneStatus);
  }, [wsData.droneStatus]);

  const cleanup = () => {
    if (animationIdRef.current) {
      cancelAnimationFrame(animationIdRef.current);
    }
    if (rendererRef.current && mountRef.current && mountRef.current.contains(rendererRef.current.domElement)) {
      mountRef.current.removeChild(rendererRef.current.domElement);
      rendererRef.current.dispose();
    }
  };

  const initThreeJS = () => {
    if (!mountRef.current) return;

    // Scene
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x87CEEB);
    sceneRef.current = scene;

    // Camera
    const camera = new THREE.PerspectiveCamera(75, 1, 0.1, 1000);
    camera.position.set(15, 15, 15);
    cameraRef.current = camera;

    // Renderer
    const renderer = new THREE.WebGLRenderer({ antialias: true });
    const container = mountRef.current;
    const width = container.clientWidth || 600;
    const height = container.clientHeight || 400;
    
    renderer.setSize(width, height);
    renderer.shadowMap.enabled = true;
    renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    rendererRef.current = renderer;

    container.appendChild(renderer.domElement);

    // Controls
    setupControls(renderer.domElement, camera);
    
    // Create scene
    createDrone();
    createEnvironment();
    animate();

    // Handle resize
    const handleResize = () => {
      if (!mountRef.current || !rendererRef.current || !cameraRef.current) return;
      
      const width = mountRef.current.clientWidth || 600;
      const height = mountRef.current.clientHeight || 400;
      
      cameraRef.current.aspect = width / height;
      cameraRef.current.updateProjectionMatrix();
      rendererRef.current.setSize(width, height);
    };

    window.addEventListener('resize', handleResize);
    
    return () => {
      window.removeEventListener('resize', handleResize);
    };
  };

  const setupControls = (domElement, camera) => {
    let isMouseDown = false;
    let mouseX = 0;
    let mouseY = 0;
    let targetX = 0;
    let targetY = 0;

    const onMouseDown = (event) => {
      isMouseDown = true;
      mouseX = event.clientX;
      mouseY = event.clientY;
    };

    const onMouseMove = (event) => {
      if (!isMouseDown) return;
      
      const deltaX = event.clientX - mouseX;
      const deltaY = event.clientY - mouseY;
      
      targetX += deltaX * 0.01;
      targetY += deltaY * 0.01;
      targetY = Math.max(-Math.PI/2, Math.min(Math.PI/2, targetY));
      
      mouseX = event.clientX;
      mouseY = event.clientY;
    };

    const onMouseUp = () => {
      isMouseDown = false;
    };

    const onWheel = (event) => {
      const radius = camera.position.length();
      const newRadius = Math.max(5, Math.min(50, radius + event.deltaY * 0.01));
      camera.position.normalize().multiplyScalar(newRadius);
    };

    domElement.addEventListener('mousedown', onMouseDown);
    domElement.addEventListener('mousemove', onMouseMove);
    domElement.addEventListener('mouseup', onMouseUp);
    domElement.addEventListener('wheel', onWheel);

    // Update camera position
    const updateCamera = () => {
      const radius = camera.position.length();
      camera.position.x = Math.cos(targetX) * Math.cos(targetY) * radius;
      camera.position.y = Math.sin(targetY) * radius;
      camera.position.z = Math.sin(targetX) * Math.cos(targetY) * radius;
      camera.lookAt(0, 0, 0);
    };

    // Store update function
    camera.userData = { update: updateCamera };
  };

  const createDrone = () => {
    const droneGroup = new THREE.Group();
    
    // Main body - more detailed
    const bodyGeometry = new THREE.BoxGeometry(1.2, 0.3, 1.2);
    const bodyMaterial = new THREE.MeshLambertMaterial({ color: 0x2c3e50 });
    const body = new THREE.Mesh(bodyGeometry, bodyMaterial);
    body.castShadow = true;
    droneGroup.add(body);

    // Camera gimbal
    const gimbalGeometry = new THREE.SphereGeometry(0.15);
    const gimbalMaterial = new THREE.MeshLambertMaterial({ color: 0x34495e });
    const gimbal = new THREE.Mesh(gimbalGeometry, gimbalMaterial);
    gimbal.position.y = -0.25;
    gimbal.castShadow = true;
    droneGroup.add(gimbal);

    // Propellers with arms
    const propPositions = [
      { x: 0.6, y: 0.4, z: 0.6, rotation: 0 },
      { x: -0.6, y: 0.4, z: 0.6, rotation: Math.PI },
      { x: 0.6, y: 0.4, z: -0.6, rotation: Math.PI },
      { x: -0.6, y: 0.4, z: -0.6, rotation: 0 }
    ];

    propPositions.forEach((pos, index) => {
      // Arm
      const armGeometry = new THREE.CylinderGeometry(0.03, 0.03, 0.8);
      const armMaterial = new THREE.MeshLambertMaterial({ color: 0x7f8c8d });
      const arm = new THREE.Mesh(armGeometry, armMaterial);
      arm.position.set(pos.x, pos.y, pos.z);
      arm.rotation.z = Math.PI / 2;
      arm.castShadow = true;
      droneGroup.add(arm);

      // Motor
      const motorGeometry = new THREE.CylinderGeometry(0.08, 0.08, 0.1);
      const motorMaterial = new THREE.MeshLambertMaterial({ color: 0x34495e });
      const motor = new THREE.Mesh(motorGeometry, motorMaterial);
      motor.position.set(pos.x, pos.y + 0.05, pos.z);
      motor.castShadow = true;
      droneGroup.add(motor);

      // Propeller
      const propGeometry = new THREE.CylinderGeometry(0.25, 0.25, 0.02);
      const propMaterial = new THREE.MeshLambertMaterial({ 
        color: 0x95a5a6,
        transparent: true,
        opacity: 0.7
      });
      const propeller = new THREE.Mesh(propGeometry, propMaterial);
      propeller.position.set(pos.x, pos.y + 0.15, pos.z);
      propeller.userData = { rotationSpeed: 0, baseRotation: pos.rotation };
      propeller.castShadow = true;
      droneGroup.add(propeller);
    });

    // Landing gear
    const gearGeometry = new THREE.CylinderGeometry(0.02, 0.02, 0.4);
    const gearMaterial = new THREE.MeshLambertMaterial({ color: 0x7f8c8d });
    
    const gearPositions = [
      { x: 0.4, z: 0.4 },
      { x: -0.4, z: 0.4 },
      { x: 0.4, z: -0.4 },
      { x: -0.4, z: -0.4 }
    ];

    gearPositions.forEach(pos => {
      const gear = new THREE.Mesh(gearGeometry, gearMaterial);
      gear.position.set(pos.x, -0.3, pos.z);
      gear.castShadow = true;
      droneGroup.add(gear);
    });

    // LED lights
    const ledGeometry = new THREE.SphereGeometry(0.03);
    const redLedMaterial = new THREE.MeshBasicMaterial({ color: 0xff0000 });
    const greenLedMaterial = new THREE.MeshBasicMaterial({ color: 0x00ff00 });
    
    const redLed = new THREE.Mesh(ledGeometry, redLedMaterial);
    redLed.position.set(0.5, 0.2, 0);
    droneGroup.add(redLed);
    
    const greenLed = new THREE.Mesh(ledGeometry, greenLedMaterial);
    greenLed.position.set(-0.5, 0.2, 0);
    droneGroup.add(greenLed);

    droneRef.current = droneGroup;
    sceneRef.current.add(droneGroup);
  };

  const createEnvironment = () => {
    // Ground with texture
    const groundGeometry = new THREE.PlaneGeometry(100, 100);
    const groundMaterial = new THREE.MeshLambertMaterial({ 
      color: 0x7cb342,
      transparent: true,
      opacity: 0.8
    });
    const ground = new THREE.Mesh(groundGeometry, groundMaterial);
    ground.rotation.x = -Math.PI / 2;
    ground.position.y = -3;
    ground.receiveShadow = true;
    sceneRef.current.add(ground);

    // Grid
    const gridHelper = new THREE.GridHelper(100, 100, 0x888888, 0xcccccc);
    gridHelper.position.y = -2.9;
    sceneRef.current.add(gridHelper);

    // Lighting setup
    const ambientLight = new THREE.AmbientLight(0x404040, 0.4);
    sceneRef.current.add(ambientLight);

    const directionalLight = new THREE.DirectionalLight(0xffffff, 0.8);
    directionalLight.position.set(20, 20, 10);
    directionalLight.castShadow = true;
    directionalLight.shadow.mapSize.width = 2048;
    directionalLight.shadow.mapSize.height = 2048;
    directionalLight.shadow.camera.near = 0.5;
    directionalLight.shadow.camera.far = 500;
    sceneRef.current.add(directionalLight);

    // Sky dome
    const skyGeometry = new THREE.SphereGeometry(200, 32, 32);
    const skyMaterial = new THREE.MeshBasicMaterial({ 
      color: 0x87CEEB,
      side: THREE.BackSide
    });
    const sky = new THREE.Mesh(skyGeometry, skyMaterial);
    sceneRef.current.add(sky);
  };

  const updateDronePosition = (droneStatus) => {
    if (!droneRef.current || !droneStatus) return;

    // Update position smoothly
    const targetPos = new THREE.Vector3(
      droneStatus.position?.x || 0,
      droneStatus.position?.z || 0,
      droneStatus.position?.y || 0
    );
    
    droneRef.current.position.lerp(targetPos, 0.1);

    // Update rotation
    if (droneStatus.attitude) {
      droneRef.current.rotation.set(
        droneStatus.attitude.roll || 0,
        droneStatus.attitude.yaw || 0,
        droneStatus.attitude.pitch || 0
      );
    }

    // Update propeller rotation
    const speed = droneStatus.armed ? 0.8 : 0.1;
    droneRef.current.children.forEach(child => {
      if (child.userData.rotationSpeed !== undefined) {
        child.rotation.y += speed;
      }
    });
  };

  const sendCommand = async (command, params = {}) => {
    try {
      console.log('Sending command:', command, params);
      
      // Update local state immediately for responsiveness
      setWsData(prev => {
        const newDroneStatus = { ...prev.droneStatus };
        
        switch (command) {
          case 'arm':
            newDroneStatus.armed = params.armed;
            newDroneStatus.flightMode = params.armed ? 'Armed' : 'Disarmed';
            break;
          case 'takeoff':
            if (newDroneStatus.armed) {
              newDroneStatus.position = { ...newDroneStatus.position, z: 3 };
              newDroneStatus.flightMode = 'Takeoff';
            }
            break;
          case 'land':
            newDroneStatus.position = { ...newDroneStatus.position, z: 0 };
            newDroneStatus.flightMode = 'Landing';
            break;
          case 'hover':
            newDroneStatus.flightMode = 'Hover';
            break;
          case 'emergency':
            newDroneStatus.armed = false;
            newDroneStatus.position = { ...newDroneStatus.position, z: 0 };
            newDroneStatus.flightMode = 'Emergency';
            break;
          case 'setAltitude':
            setTargetAltitude(params.altitude);
            newDroneStatus.position = { ...newDroneStatus.position, z: params.altitude };
            break;
        }
        
        return { ...prev, droneStatus: newDroneStatus };
      });
      
      // In futuro, inviare il comando al backend
      // await fetch('/api/v1/drone/command', { ... });
      
    } catch (error) {
      console.error('Error sending command:', error);
    }
  };

  const animate = () => {
    animationIdRef.current = requestAnimationFrame(animate);
    
    if (cameraRef.current && cameraRef.current.userData.update) {
      cameraRef.current.userData.update();
    }
    
    if (rendererRef.current && sceneRef.current && cameraRef.current) {
      rendererRef.current.render(sceneRef.current, cameraRef.current);
    }
  };

  const droneStatus = wsData.droneStatus;

  return (
    <Grid container spacing={3}>
      {/* 3D Visualization */}
      <Grid item xs={12} lg={8}>
        <Paper elevation={3} sx={{ p: 2 }}>
          <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
            <Typography variant="h6">
              Drone 3D Visualization
            </Typography>
            <Box display="flex" gap={1}>
              <Chip 
                label={droneStatus.flightMode} 
                color={droneStatus.armed ? "success" : "default"}
                size="small"
              />
              <Chip 
                label={droneStatus.connected ? "Connected" : "Disconnected"}
                color={droneStatus.connected ? "success" : "error"}
                size="small"
              />
            </Box>
          </Box>
          
          <Box 
            ref={mountRef}
            sx={{ 
              width: '100%', 
              height: '500px',
              border: '2px solid #e0e0e0',
              borderRadius: 2,
              position: 'relative',
              overflow: 'hidden',
              background: 'linear-gradient(to bottom, #87CEEB, #98FB98)'
            }}
          />
          
          {!droneStatus.connected && (
            <Alert severity="warning" sx={{ mt: 1 }}>
              Drone disconnected - showing last known position
            </Alert>
          )}
        </Paper>
      </Grid>

      {/* Control Panel */}
      <Grid item xs={12} lg={4}>
        <Grid container spacing={2}>
          {/* Flight Controls */}
          <Grid item xs={12}>
            <Paper elevation={3} sx={{ p: 2 }}>
              <Typography variant="h6" gutterBottom>
                Flight Controls
              </Typography>
              
              <Box mb={2}>
                <FormControlLabel
                  control={
                    <Switch
                      checked={droneStatus.armed}
                      onChange={(e) => sendCommand('arm', { armed: e.target.checked })}
                      color="error"
                    />
                  }
                  label="Armed"
                />
              </Box>

              <ButtonGroup variant="contained" fullWidth sx={{ mb: 2 }}>
                <Button 
                  color="success"
                  onClick={() => sendCommand('takeoff')}
                  disabled={!droneStatus.armed}
                >
                  Takeoff
                </Button>
                <Button 
                  color="primary"
                  onClick={() => sendCommand('hover')}
                >
                  Hover
                </Button>
                <Button 
                  color="warning"
                  onClick={() => sendCommand('land')}
                >
                  Land
                </Button>
              </ButtonGroup>

              <Button 
                variant="contained" 
                color="error"
                fullWidth
                onClick={() => sendCommand('emergency')}
                sx={{ mb: 2 }}
              >
                Emergency Stop
              </Button>

              <Divider sx={{ my: 2 }} />

              <Typography variant="body2" gutterBottom>
                Target Altitude: {droneStatus.position?.z?.toFixed(2) || 0}m
              </Typography>
              <Slider
                value={droneStatus.position?.z || 0}
                min={0}
                max={20}
                step={0.5}
                onChange={(e, value) => sendCommand('setAltitude', { altitude: value })}
                valueLabelDisplay="auto"
                valueLabelFormat={(value) => `${value}m`}
                disabled={!droneStatus.armed}
                marks={[
                  { value: 0, label: '0m' },
                  { value: 10, label: '10m' },
                  { value: 20, label: '20m' }
                ]}
              />
            </Paper>
          </Grid>

          {/* Telemetry */}
          <Grid item xs={12}>
            <Paper elevation={3} sx={{ p: 2 }}>
              <Typography variant="h6" gutterBottom>
                Telemetry Data
              </Typography>
              
              <Box mb={2}>
                <Typography variant="subtitle2" color="text.secondary">
                  Position (m)
                </Typography>
                <Typography variant="body2" component="div">
                  X: {droneStatus.position?.x?.toFixed(2) || 0} | 
                  Y: {droneStatus.position?.y?.toFixed(2) || 0} | 
                  Z: {droneStatus.position?.z?.toFixed(2) || 0}
                </Typography>
              </Box>
              
              <Box mb={2}>
                <Typography variant="subtitle2" color="text.secondary">
                  Attitude (rad)
                </Typography>
                <Typography variant="body2" component="div">
                  Roll: {droneStatus.attitude?.roll?.toFixed(3) || 0} | 
                  Pitch: {droneStatus.attitude?.pitch?.toFixed(3) || 0} | 
                  Yaw: {droneStatus.attitude?.yaw?.toFixed(3) || 0}
                </Typography>
              </Box>
              
              <Box mb={2}>
                <Typography variant="subtitle2" color="text.secondary">
                  Battery Status
                </Typography>
                <Box display="flex" alignItems="center" mb={1}>
                  <Typography variant="body2" sx={{ minWidth: '60px' }}>
                    {droneStatus.battery?.percentage?.toFixed(1) || 0}%
                  </Typography>
                  <LinearProgress 
                    variant="determinate" 
                    value={droneStatus.battery?.percentage || 0}
                    color={
                      (droneStatus.battery?.percentage || 0) > 50 
                        ? "success" 
                        : (droneStatus.battery?.percentage || 0) > 20 
                          ? "warning" 
                          : "error"
                    }
                    sx={{ flexGrow: 1, mx: 1, height: 8, borderRadius: 4 }}
                  />
                  <Typography variant="caption">
                    {droneStatus.battery?.voltage?.toFixed(2) || 0}V
                  </Typography>
                </Box>
              </Box>

              <Divider sx={{ my: 2 }} />

              <Box display="flex" gap={1} flexWrap="wrap">
                <Chip 
                  label={droneStatus.armed ? "Armed" : "Disarmed"}
                  color={droneStatus.armed ? "error" : "default"}
                  size="small"
                />
                <Chip 
                  label={droneStatus.flightMode}
                  color="primary"
                  size="small"
                />
                <Chip 
                  label={`${droneStatus.position?.z?.toFixed(1) || 0}m AGL`}
                  color="info"
                  size="small"
                />
              </Box>
            </Paper>
          </Grid>
        </Grid>
      </Grid>
    </Grid>
  );
};

export default DroneControlDashboard;