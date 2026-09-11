# Chapter 28 Mac Acquisition and Internals

The proliferation of systems running Mac OS X in both home and corporate environments has resulted in Mac systems being a focus of targeted attacks. Driven by these factors, the forensics community has worked to develop tools for Mac systems that are on par with the robust investigative capabilities currently available for Windows and Linux systems. To prepare you for Mac memory forensics, this chapter introduces some of the unique facets of the Mac operating system, such as 64-bit addressing on 32-bit kernels, the atypical userland and kernel address space layouts, and the use of microkernel components. Additionally, you’ll learn how to build Volatility profiles for Mac systems and which tools to use for memory acquisition.

**NOTE**

Part IV of this book focuses heavily on in-memory artifacts of Mac systems. If you would like to learn about advances in Mac disk forensics and malware anaysis, we suggest reading the presentations indexed on Sarah Edwards’ website ([http://mac4n6.com/](http://mac4n6.com/)).

## Mac Design

If you read Part III, “Linux Memory Forensics,” you are now very familiar with how Linux was designed and organized in both the kernel and userland. As you will soon see, Mac is very similar to Linux because it is heavily based on Berkeley Software Distribution (BSD). Both BSD and Linux were influenced by the initial designs and philosophies of Unix. The similarities between these operating systems result in a substantially smaller learning curve ...
