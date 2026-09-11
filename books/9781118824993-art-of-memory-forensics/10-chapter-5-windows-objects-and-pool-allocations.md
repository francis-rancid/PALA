# Chapter 5 Windows Objects and Pool Allocations

All the artifacts that you find in memory dumps share a common origin: They all start out as an allocation. How, when, and why the memory regions were allocated sets them apart, in addition to the actual data stored within and around them. From a memory forensics perspective, studying these characteristics can help you make inferences about the content of an allocation, leading to your ability to find and label specific types of data throughout a large memory dump. Furthermore, becoming familiar with the operating system’s algorithms for allocation and de-allocation of memory can help you understand the context of data when you find it—for example, whether it is currently in use or marked as free.

This chapter introduces you to the concepts of Windows executive objects, kernel pool allocations, and pool tag scanning. Specifically, you will use this knowledge to find objects (such as processes, files, and drivers) by using a method that is independent of how the operating system enumerates the objects. Thus, you can defeat rootkits that try to hide by manipulating the operating system’s internal data structures. Furthermore, you can identify objects that were used but have since been discarded (but not overwritten), giving you valuable insight into events that occurred in the past.

## Windows Executive Objects

A great deal of memory forensics involves finding and analyzing executive objects. In Chapter 2, you learned that Windows is ...
