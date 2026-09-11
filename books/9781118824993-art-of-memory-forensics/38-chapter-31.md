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
