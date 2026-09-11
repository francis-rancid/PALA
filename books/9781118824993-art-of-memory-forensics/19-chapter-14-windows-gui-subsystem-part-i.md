# Chapter 14 Windows GUI Subsystem, Part I

The Windows graphical user interface (GUI) subsystem is responsible for managing user input, such as mouse movements and keystrokes. In addition, it draws the display surface; presents windows, buttons, and menus; and provides the necessary isolation to support multiple concurrent users logged in via the console, RDP, and Fast-User Switching. The GUI subsystem plays a huge role in everyday computer use, and it is inevitable that malware and attackers unknowingly modify GUI memory during the course of their actions. Unfortunately, there are few tools, much less forensic tools, capable of analyzing and reporting on artifacts created in and maintained by this subsystem.

The next two chapters introduce a collection of data structures, classes, algorithms, APIs, and plugins for extracting GUI-related evidence from physical memory (RAM) of 32- and 64-bit Windows XP, Server 2003, Vista, Server 2008, and Windows 7. We will be discussing various specific examples of how malicious code can be detected in memory and how you can apply knowledge of the GUI internals to forensic investigations.

## The GUI Landscape

The GUI subsystem is composed of various objects that all work together to provide an enhanced user experience. The relationship of these components is summarized in [Figure 14-1](#figure14-1). The diagram does not capture all the GUI internals—only the most important ones for forensics and malware investigations.

*[**Figure 14-1:**](#figureanchor14-1) Windows GUI landscape*
