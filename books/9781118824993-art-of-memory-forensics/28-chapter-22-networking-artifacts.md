# Chapter 22 Networking Artifacts

After a network breach, the first questions that must be answered are often the following: Which system was initially infected, which machines were later compromised through lateral movement, and which remote systems were involved in data exfiltration or command and control? Memory forensics is critical to answering these questions because very few of the related artifacts are written to disk. In this chapter, you will learn how this data is stored within Linux memory samples, what you can do to recover it, and how to draw conclusions based on what you find.

## Network Socket File Descriptors

Before you can begin to analyze network information in memory, you must first locate the network socket file descriptors. Because a wide range of items (open file handles, network sockets, pipes, etc.) are represented as file descriptors, Linux provides a common application programming interface (API) for accessing them. By leveraging the data structures of this generic API, you can successfully determine the purpose of a file descriptor.

## **Analysis Objectives**

Your objectives are these:

- **Identify socket file descriptors:** In the previous chapter, you learned how to enumerate a process’ file descriptors. Now, you will learn how to determine which descriptors belong to network sockets.
- **Understand socket operations**: You will learn how the operations structures of a socket affect how the kernel interacts with the socket and processes its data.

## **Data Structures**
