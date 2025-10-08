/**
 * API utility functions with timeout support
 */

// Default timeout in milliseconds
const DEFAULT_TIMEOUT = 10000; // 10 seconds

/**
 * Create an AbortController with timeout
 * @param {number} timeout - Timeout in milliseconds
 * @returns {Object} - Object with controller and timeoutId
 */
export const createTimeoutController = (timeout = DEFAULT_TIMEOUT) => {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeout);
  
  return { controller, timeoutId };
};

/**
 * Fetch with timeout support
 * @param {string} url - Request URL
 * @param {Object} options - Fetch options
 * @param {number} timeout - Timeout in milliseconds
 * @returns {Promise} - Fetch promise
 */
export const fetchWithTimeout = async (url, options = {}, timeout = DEFAULT_TIMEOUT) => {
  const { controller, timeoutId } = createTimeoutController(timeout);
  
  try {
    const response = await fetch(url, {
      ...options,
      signal: controller.signal
    });
    
    clearTimeout(timeoutId);
    return response;
  } catch (error) {
    clearTimeout(timeoutId);
    
    if (error.name === 'AbortError') {
      throw new Error(`Request timeout after ${timeout}ms`);
    }
    
    throw error;
  }
};

/**
 * API call with timeout and error handling
 * @param {string} url - Request URL
 * @param {Object} options - Fetch options
 * @param {number} timeout - Timeout in milliseconds
 * @returns {Promise} - API response
 */
export const apiCall = async (url, options = {}, timeout = DEFAULT_TIMEOUT) => {
  try {
    const response = await fetchWithTimeout(url, options, timeout);
    
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }
    
    return response;
  } catch (error) {
    console.error(`API call failed for ${url}:`, error);
    throw error;
  }
};

/**
 * GET request with timeout
 * @param {string} url - Request URL
 * @param {number} timeout - Timeout in milliseconds
 * @returns {Promise} - JSON response
 */
export const apiGet = async (url, timeout = DEFAULT_TIMEOUT) => {
  const response = await apiCall(url, { method: 'GET' }, timeout);
  return response.json();
};

/**
 * POST request with timeout
 * @param {string} url - Request URL
 * @param {Object} data - Request body
 * @param {number} timeout - Timeout in milliseconds
 * @returns {Promise} - JSON response
 */
export const apiPost = async (url, data, timeout = DEFAULT_TIMEOUT) => {
  const response = await apiCall(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(data)
  }, timeout);
  return response.json();
};

/**
 * PUT request with timeout
 * @param {string} url - Request URL
 * @param {Object} data - Request body
 * @param {number} timeout - Timeout in milliseconds
 * @returns {Promise} - JSON response
 */
export const apiPut = async (url, data, timeout = DEFAULT_TIMEOUT) => {
  const response = await apiCall(url, {
    method: 'PUT',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(data)
  }, timeout);
  return response.json();
};

/**
 * DELETE request with timeout
 * @param {string} url - Request URL
 * @param {number} timeout - Timeout in milliseconds
 * @returns {Promise} - JSON response
 */
export const apiDelete = async (url, timeout = DEFAULT_TIMEOUT) => {
  const response = await apiCall(url, { method: 'DELETE' }, timeout);
  return response.json();
};

/**
 * Multiple API calls with timeout
 * @param {Array} requests - Array of request objects
 * @param {number} timeout - Timeout in milliseconds
 * @returns {Promise} - Array of responses
 */
export const apiAll = async (requests, timeout = DEFAULT_TIMEOUT) => {
  const promises = requests.map(request => {
    if (typeof request === 'string') {
      return apiGet(request, timeout);
    }
    return apiCall(request.url, request.options, timeout);
  });
  
  return Promise.all(promises);
};

/**
 * Retry API call with exponential backoff
 * @param {Function} apiFunction - API function to retry
 * @param {number} maxRetries - Maximum number of retries
 * @param {number} baseDelay - Base delay in milliseconds
 * @returns {Promise} - API response
 */
export const apiRetry = async (apiFunction, maxRetries = 3, baseDelay = 1000) => {
  let lastError;
  
  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      return await apiFunction();
    } catch (error) {
      lastError = error;
      
      if (attempt === maxRetries) {
        break;
      }
      
      const delay = baseDelay * Math.pow(2, attempt);
      await new Promise(resolve => setTimeout(resolve, delay));
    }
  }
  
  throw lastError;
};
