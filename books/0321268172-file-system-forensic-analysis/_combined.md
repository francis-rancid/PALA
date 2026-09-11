## Foreword

Computer forensics is a relatively new field, and over the years it has been called many things: “computer forensics,” “digital forensics,” and “media analysis” to name a few. It has only been in the past few years that we have begun to recognize that all of our digital devices leave digital breadcrumbs and that these breadcrumbs are valuable evidence in a wide range of inquiries. While criminal justice professionals were some of the first to take an interest in this digital evidence, the intelligence, information security, and civil law fields have enthusiastically adopted this new source of information.

Digital forensics has joined the mainstream. In 2003, the American Society of Crime Laboratory Directors–Laboratory Accreditation ...
## Preface

One of the biggest challenges that I have faced over the years while developing *The Sleuth Kit* (TSK) has been finding good file and volume system (such as partition tables, RAID, and so on) documentation. It also has been challenging to explain to users why certain files cannot be recovered or what to do when a corrupt file system is encountered because there are no good references to recommend. It is easy to find resources that describe file systems at a high level, but source code is typically needed to learn the details. My goal for this book is to fill the void and describe how data are stored on disk and describe where and how digital evidence can be found.

There are two target audiences for this book. One is the experienced investigator ...
## Acknowledgments

I would like to thank many people for helping me with digital forensics. First, thanks go out to those who have helped me in general over the years. My appreciation goes to Eoghan Casey, Dave Dittrich, Dan Farmer, Dan Geer, Dan Kalil, Warren Kruse, Gary Palmer, Eugene Spafford, Lance Spitzner, and Wietse Venema for various forms of guidance, knowledge, and opportunities.

I would also like to thank Cory Altheide, Eoghan Casey, Knut Eckstein, and Jim Lyle for reviewing the entire book. Special thanks go to Knut, who went through every hexdump dissection of the example disk images and verified each hexadecimal to decimal conversion (and found several typos), and to Eoghan for reminding me when the content needed more practical applications. ...
# Part I: Foundations
## 1. Digital Investigation Foundations

I am going to assume that anyone interested in this book does not need motivation with respect to why someone would want to investigate a computer or other digital device, so I will skip the customary numbers and statistics. This book is about how you can conduct a smarter investigation, and it is about data and how they are stored. Digital investigation tools have become relatively easy to use, which is good because they reduce the time needed to conduct an investigation. However, it also means that the investigator may not fully understand the results. This could be dangerous when the investigator needs to testify about the evidence and from where it came. This book starts with the basic foundations of investigations ...
## 2. Computer Foundations

The goal of this chapter is to cover the low-level basics of how computers operate. In the following chapters of this book, we examine, in detail, how data are stored, and this chapter provides background information for those who do not have programming or operating system design experience. This chapter starts with a discussion about data and how they are organized on disk. We discuss binary versus hexadecimal values and little-and big-endian ordering. Next, we examine the boot process and code required to start a computer. Lastly, we examine hard disks and discuss their geometry, ATA commands, host protected areas, and SCSI.

### Data Organization

The purpose of the devices we investigate is to process digital data, so ...
## 3. Hard Disk Data Acquisition

The bulk of this book deals with the analysis of data found on a storage device, namely a hard disk. Data can be analyzed on a live system, but it is more common to acquire a copy of the data for a dead analysis. Acquisition typically occurs during the System Preservation phase of an investigation and is one of the most important phases in a digital forensic investigation because if data are not collected from the system, they could be lost and therefore not recognized as evidence. Further, if data are not collected properly, their value as legal evidence is diminished. This chapter shows the theory of how hard disk data can be acquired and includes a case study using the Linux `dd` tool.

### Introduction

We saw in
# Part II: Volume Analysis
## 4. Volume Analysis

This chapter begins [Part 2](part02.html#part02), “[Volume Analysis](part02.html#part02),” of the book, and we are going to now discuss volume analysis. Volume analysis involves looking at the data structures that are involved with partitioning and assembling the bytes in storage devices so that we get volumes. Volumes are used to store file system and other structured data, which we will analyze in [Part 3](part03.html#part03), “[File System Analysis](part03.html#part03),” of the book. This chapter takes an abstract approach to the basic concepts of volume analysis and discusses the principles that apply to all types of volume systems. The next three chapters will focus on specific types of partitioning and assembly systems.

### Introduction

Digital storage media is organized to allow efficient retrieval of data. ...
## 5. PC-based Partitions

The last chapter provided an overview of volume analysis and why it’s important. Now we’re going to leave the abstract discussion of volumes and dive into the details of the partition systems used in personal computers. In this chapter, we will look at DOS partitions, Apple partitions, and removable media. For each system, we review how it works and look at its data structure. If you are not interested in the data structure details, you can skip those sections. This chapter also covers special considerations that should be made when analyzing these systems. The next chapter will examine server-based partitioning systems.

### DOS Partitions

The most commonly encountered partition system is the DOS-style partition. DOS partitions ...
## 6. Server-based Partitions

In the last chapter we saw how personal computers partition their storage volumes, and now we are going to look at how some of the servers partition their volumes. The basic concepts of this chapter and the previous chapter are identical. In fact, they are separated only because I needed some way to break the content into medium-sized chapters, and this separation seemed as good as anything else (even though DOS and Apple partitions are also used in servers). In this chapter, we are going to look at FreeBSD, NetBSD, and OpenBSD partition systems; Sun Solaris partition systems; and GPT partitions that are found in 64-bit Intel Itanium systems.

### BSD Partitions

It is becoming more common for computer investigations to ...
## 7. Multiple Disk Volumes

In many critical servers, multiple disks are used for performance, reliability, or scalability. The disks are merged and processed so that they look normal but they are not. This chapter covers RAID and disk spanning systems, both of which can be difficult to investigate. There can be many challenges when investigating a system that uses a multiple disk volume, and not all the problems have been solved. This chapter explains the technology behind both of these volume systems and then provides some suggestions for analyzing or acquiring the data. Of any chapter in this book, this will likely become outdated the most quickly because new technology is being developed to create new types of storage systems and because new ...
# Part III: File System Analysis
## 8. File System Analysis

File system analysis examines data in a volume (i.e., a partition or disk) and interprets them as a file system. There are many end results from this process, but examples include listing the files in a directory, recovering deleted content, and viewing the contents of a sector. Recall that analyzing the contents of a file is application-level analysis and is not covered in this book. In this chapter, we look at the general design of file systems and different analysis techniques. This chapter approaches the topic in an abstract fashion and is not limited to how a specific tool analyzes a file system. Instead, we discuss the analysis in general terms. The remaining nine chapters discuss how specific file systems are designed ...
## 9. FAT Concepts and Analysis

The *File Allocation Table* (FAT) file system is one of the most simple file systems found in common operating systems. FAT is the primary file system of the Microsoft DOS and Windows 9x operating systems, but the NT, 2000, and XP line has defaulted to the New Technologies File System (NTFS), which is discussed later in the book. FAT is supported by all Windows and most Unix operating systems and will be encountered by investigators for years to come, even if it is not the default file system of desktop Windows systems. FAT is frequently found in compact flash cards for digital cameras and USB “thumb drives.” Many people are familiar with the basic concepts of the FAT file system but may not be aware of data hiding ...
## 10. FAT Data Structures

In the previous chapter, we examined the basic concepts of a FAT file system and how to analyze it. Now we are going to get more detailed and examine the data structures that make up FAT. This chapter will ignore the five-category model and instead focus on each individual data structure. This makes it easier to understand FAT because many of the data structures are in more than one category. I assume that you have already read [Chapter 9](ch09.html#ch09), “[FAT Concepts and Analysis](ch09.html#ch09),” or that you are reading it in parallel. All the hexdumps and data shown in this chapter correspond to the data that were analyzed in [Chapter 9](ch09.html#ch09) using tools from *The Sleuth Kit* (TSK).

### Boot Sector

The boot sector is located in the first sector of FAT file system ...
## 11. NTFS Concepts

The *New Technologies File System* (NTFS) was designed by Microsoft and is the default file system for Microsoft Windows NT, Windows 2000, Windows XP, and Windows Server. At the time of this writing, Microsoft has discontinued the sale of the Windows 98 and ME lines, and the home version of Windows XP is standard among new consumer systems. FAT will still exist in mobile and small storage devices, but NTFS will likely be the most common file system for Windows investigations. NTFS is a much more complex file system than FAT because it has many features and is very scalable. Because of the complexity of NTFS, we will need three chapters to discuss it. This chapter discusses the core concepts of NTFS that apply to all the five categories ...
## 12. NTFS Analysis

This is the second NTFS chapter, and we will now start to discuss analysis techniques and considerations using the five-category model presented in [Chapter 8](ch08.html#ch08), “[File System Analysis](ch08.html#ch08).” NTFS is much different from other file systems, so we covered the core NTFS concepts in the previous chapter before diving into this material. If you are not familiar with NTFS and skipped [Chapter 11](ch11.html#ch11), I recommend returning to it before starting this chapter. [Chapter 13](ch13.html#ch13), “[NTFS Data Structures](ch13.html#ch13),” covers the data structures for NTFS. Most of this book has been organized so that you can read the file system analysis and data structure chapters in parallel. This is more difficult with NTFS because everything is a file, and it is difficult to show the ...
## 13. NTFS Data Structures

This is the third and final chapter devoted to NTFS, and here we will examine its data structures. The previous two chapters examined the basic concepts of NTFS and how to analyze it. For many, the information covered thus far is sufficient, but others of us want to know more about what is going on. This chapter is organized so that we cover the data structures of the basic elements first and then examine the specific attributes and index types. Lastly, the file system metadata files are covered. Unlike the other file system chapters, this one was written so that it should be read after [Chapter 11](ch11.html#ch11), “[NTFS Concepts](ch11.html#ch11),” and [Chapter 12](ch12.html#ch12), “[NTFS Analysis](ch12.html#ch12).” The first part of the chapter can be read in parallel with [Chapter 11](ch11.html#ch11)
## 14. Ext2 and Ext3 Concepts and Analysis

The Ext2 and Ext3 file systems, which I will lump into the term ExtX from now on, are the default file systems for many distributions of the Linux operating system. Ext3 is the newer version of Ext2 and adds file system journaling, but the basic Ext2 construction remains the same. ExtX are based on the *UNIX File System* (UFS). ExtX removed many of the components of UFS that are no longer needed, so it is easier to understand and explain and will be covered first. Unlike other operating systems, Linux supports a large number of file systems, and each distribution can choose which file system will be the default. At the time of this writing, Ext3 is the default file system of most distributions, but the Reiser ...
## 15. Ext2 and Ext3 Data Structures

The previous chapter outlined the concepts behind the design of ExtX, and this chapter describes the various data structures that make up an ExtX file system. It is assumed that you have either read the previous chapter or are reading it in parallel with this one. If you are not interested in learning about the data structures and layout, you can safely skip this chapter.

### Superblock

ExtX uses the superblock to store the basic file system category data. It is located 1,024 bytes from the start of the file system and has 1,024 bytes allocated to it. It has the fields given in [Table 15.1](ch15.html#ch15tab01).

![Image](/api/v2/epubs/urn:orm:book:0321268172/files/graphics/15tab01.jpg)

**Table 15.1** Data structure ...
## 16. UFS1 and UFS2 Concepts and Analysis

The *Unix File System* (UFS) comes in several variations and can be found in many types of UNIX systems, including FreeBSD, HP-UX, NetBSD, OpenBSD, Apple OS X, and Sun Solaris. Many OSes have modified one or more data structures over the years to suit their needs, but they all have the same concepts. Currently, the two major variations are UFS1 and UFS2. UFS2 supports larger disks and larger time stamps. I will use the term UFS to refer to both file systems. An investigator might encounter a UFS file system when investigating a Unix system, typically a server. Ext2 and Ext3 are based on UFS, and because they were already discussed in detail, this chapter will be briefer and assume that you understand the ...
## 17. UFS1 and UFS2 Data Structures

This chapter describes the data structures that make up a UFS1 or UFS2 file system. The general concepts and analysis techniques for UFS were discussed in the previous chapter, and this chapter shows the layout of the data structures and where they are located in an example file system image. It is assumed that you are reading this chapter in parallel with the previous chapter or that you have already read it. As mentioned in the previous chapter, the UFS data structures contain multiple fields to store a single value in different formats. For example, the size of a block is stored both as a number of fragments and as a number of bytes. The different formats prevent the OS from having to calculate the different ...
## A. The Sleuth Kit and Autopsy

The Sleuth Kit (TSK) and the Autopsy Forensic Browser are open source Unix-based tools that I first released (in some form) in early 2001. TSK is a collection of over 20 command line tools that can analyze disk and file system images for evidence. To make the analysis easier, the Autopsy Forensic Browser can be used. Autopsy is a front end to the TSK tools and provides a point-and-click type of interface.

This appendix gives more details about TSK and Autopsy. TSK is used throughout this book in the examples, but this is the only place that describes how you can use it. Both Autopsy and TSK can be downloaded for free from `http://www.sleuthkit.org`.

The Web site also contains information about e-mail lists for tool ...
![Image](/api/v2/epubs/urn:orm:book:0321268172/files/graphics/bmfig01.jpg)
![Image](/api/v2/epubs/urn:orm:book:0321268172/files/graphics/bmfig02.jpg)
