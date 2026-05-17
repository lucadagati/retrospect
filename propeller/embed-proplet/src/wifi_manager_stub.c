/* Stub WiFi manager for boards without CONFIG_WIFI (e.g. native_sim). */
#include "wifi_manager.h"

void wifi_manager_init(void) {}

int wifi_manager_connect(const char *ssid, const char *psk)
{
	(void)ssid;
	(void)psk;
	return 0;
}

int wifi_manager_enable_ap(const char *ssid, const char *psk,
			   const char *ip_address, const char *netmask)
{
	(void)ssid;
	(void)psk;
	(void)ip_address;
	(void)netmask;
	return 0;
}
