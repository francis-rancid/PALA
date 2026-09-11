# Chapter 12 Windows Services

Services on Windows are usually noninteractive (they do not directly accept user input), run consistently in the background, and often run with higher privileges than most programs users launch. Examples of services include the event-logging facility, the print spooler, the host firewall, and the time daemon. Many antivirus products, including Microsoft’s own Windows Defender and Security Center, run as services. Additionally, malicious code and adversaries often leverage services for persistence (to survive reboots), to load kernel drivers, and to blend in with legitimate components of the system.

This chapter introduces you to the internals of Windows services and shows how this knowledge can help you investigate compromised systems. It explains the major advantages to extracting service-related information from RAM rather than relying on only data from the registry. To demonstrate the concepts, you’ll examine several scenarios involving malware such as Conficker, TDL3, Blazgel, and the tools adversaries use such as the Comment Crew (also known as APT1).

## Service Architecture

The diagram in [Figure 12-1](#figure12-1) shows how the key components of the Windows service architecture work together. A list of installed services and their configurations is stored in the registry under the `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\services` key. Each service has a dedicated subkey with various values that describe how and when the service starts; whether the service ...
