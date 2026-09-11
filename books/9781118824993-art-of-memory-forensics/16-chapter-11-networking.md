# Chapter 11 Networking

Almost all malware has some sort of networking capability, whether the purpose is to contact a command and control server, spread to other machines, or create a backdoor on the system. Because the Windows OS must maintain state and pass packets it receives to the correct process or driver, it is no surprise that the involved API functions result in the creation of significant artifacts in memory. Additionally, attackers, whether remote or local, inevitably leave traces of their network activities in web browser histories, DNS caches, and so on.

This chapter provides you with an understanding of how network artifacts are created in memory and which factors are most important to your investigation. Also, you learn the significance of Microsoft fully redesigning the TCP/IP stack starting with Windows Vista; and you’ll explore two undocumented methods of recovering sockets and connections from memory dumps. Furthermore, you’ll discover why responding quickly to potential incidents is paramount, and why correlating network-related evidence in memory with external data sources such as packet captures and firewall/proxy/IDS logs is invaluable.

## Network Artifacts

The two primary types of network artifacts are sockets and connections. Sockets define endpoints for communications. Applications create *client* sockets to initiate connections to remote servers and they create *server* sockets to listen on an interface for incoming connections. You have a few ways to create ...
