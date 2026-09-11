# Preface

The enormous popularity of the World Wide Web in the early 1990s demonstrated the commercial potential of offering multimedia resources through digital networks. Representation of media in digital format facilitates its access. Digital media includes text, digital audio, images, video, and software. The recent growth of networked multimedia systems has increased the need for the protection of digital media. Since commercial interests seek to use the digital networks to offer digital media for profit, it is particularly important for the protection and enforcement of intellectual property rights, and they have a strong interest in protecting their ownership rights. On the other hand, the age of digital multimedia has brought many advantages ...
Chapter 1
# Introduction

## Abstract

With the rapid development of digital image processing techniques and the popularization of the Internet, digital images are being used more and more widely in a number of applications from medical imaging and law enforcement to banking and daily consumer use. They are much more convenient in editing, copying, storing, transmitting, and utilizing. However, unfortunately, at the same time, digital images are facing the problems of copyright protection and content authentication. Since the 1990s, the development of information hiding techniques has provided a way to protect digital media. Such techniques embed some secret information like private annotations, business logos, and critical intelligence into cover media ...
## 1.2. Overview of Information Hiding

Digital equipment and computer technologies provide a great convenience for production and access to multimedia information such as audio, images, video, animation, text, and 3D models. Meanwhile, with the growing popularity of the Internet, multimedia information exchange has reached unprecedented breadth and depth. People can now publish their works through the Internet, communicate important information, or perform network trade. However, as a result, some very serious problems also emerge: it becomes easier to infringe the rights of digital works and violate the privacy of people, it becomes more convenient to tamper and forge the digital works, and the malicious attacks become more rampant. Therefore, the ...
## 1.3. Overview of Image Coding and Compression Techniques

### 1.3.1. Source Coding and Data Compression

In computer science and information theory, data compression or source coding is the process of encoding information with fewer bits than an unencoded representation would use based on specific encoding schemes. As with any communication, compressed data communication only works when both the sender and receiver of the information understand the encoding scheme. Similarly, compressed data can only be understood if the decoding method is known by the receiver. Compression is useful because it helps reduce the consumption of expensive resources, such as the hard disk space or the transmission bandwidth. On the downside, compressed data must be decompressed ...
## 1.4. Overview of Information Hiding Techniques for Images

In this section, we provide an overview the information hiding techniques for images. That is, the cover object is a digital image, and after we embed the secret information into the cover image, we can get the stego image. According to different purposes and applications, we can mainly classify information hiding techniques into four categories, i.e., robust watermarking for copyright protection, fragile watermarking for content authentication, fingerprinting for transaction tracking, and steganography for covert communication. For these four categories, we do not care regarding the recoverability of the cover image after extracting the secret information in the stego image. For some areas, ...
## 1.5. Applications of Lossless Information Hiding in Images

Lossless information hiding in images is gaining more attention in the past few years because of its increasing applications in military communication, health care, and law enforcement. Following are some typical application fields.
### 1.5.1. Lossless Hiding Authentication

People can use lossless reversible watermarking algorithms to achieve the lossless watermark authentication, supporting completely accurate authentication for the cover media, which is actually the original intention of reversible watermarking schemes. A more comprehensive and detailed discussion of this aspect can be found in Fridrich's article [60], and we will not repeat them here.From the idea of the reversible watermarking ...
## 1.6. Main Content of This Book

This book focuses on lossless information hiding techniques for images, which are classified into three categories, i.e., spatial domain–based schemes, transform domain–based schemes, and compressed domain–based schemes. For compressed domain–based schemes, we mainly focus on four compressed domains, i.e., VQ, BTC, JPEG, and JPEG2000. Succedent chapters are organized as follows:Chapter 2 discusses the lossless information techniques in the spatial domain. In this chapter, we first provide an overview the spatial domain–based lossless schemes. Then, we discuss some typical spatial domain lossless information hiding methods, including modulo addition–based schemes, DE-based schemes, histogram modification–based schemes, ...
Chapter 2
# Lossless Information Hiding in Images on the Spatial Domain

## Abstract

Unlike traditional information hiding techniques, the features of lossless information hiding techniques are: (1) by using the lossless embedding model, the secret information is embedded in digital media to achieve copyright protection or content authentication, and (2) after extracting the secret information, the digital media can be recovered losslessly to protect the integrity of the original digital media. These techniques therefore attract the interest of academicians and show broad application prospects in medical images and other sensitive applications. Lossless information hiding algorithms for images can be classified into three categories, i.e., the spatial ...
## 2.2. Modulo Addition–Based Scheme

The modulo addition–based robust reversible watermarking algorithm first appeared in the patent of Eastman Kodak Company applied by Honsinger et al. [4]; then Fridrich et al. [6] extended this idea to the DCT transform domain. The modulo addition idea can achieve reversibility, because the nonadaptive watermark signal can be directly subtracted from the pixel value without any loss, as long as we know the possible embedded watermark pattern and the pixel value overflow problem does not appear, and this modulo addition operation is completely reversible. In Refs. [4,6], this modulo addition mode is used for embedding authentication information into the carrier media. The watermark embedding and authentication process ...
## 2.3. Difference Expansion–Based Schemes

### 2.3.1. Tian's Scheme

The DE technique for reversible data hiding was first proposed by Tian and Tiara [9,10]. It is a high-capacity approach based on expanding the pixel difference value between neighboring pixels. His method allows one bit to be embedded in every pair of pixels. Given a pair of 256-grayscale image pixel values (x, y), 0≤x, y≤255, their integer average c and difference d are computed asc=⌊x+y2⌋,d=x−y

![image](/api/v2/epubs/urn:orm:book:9780128121665/files/IMAGES/B9780128120064000024/epub.stripin/si5.png) (2.3)

where ⌊z⌋![image](/api/v2/epubs/urn:orm:book:9780128121665/files/IMAGES/B9780128120064000024/main.stripin/si6.png)

 denotes the floor function that seeks the greatest integer less than or equal ...
## 2.4. Histogram Modification–Based Schemes

### 2.4.1. Original Histogram Shifting–Based Scheme

Ni et al. [22] proposed a novel reversible algorithm based on histogram shifting techniques. The algorithm first finds a zero point (no pixel) and a peak point (a maximum number of pixels) of the image histogram. If zero point does not exist for some image histogram, a minimum point with a minimum number of pixels is treated as a zero point by memorizing the pixel grayscale value and the coordinates of those pixels as overhead information. Ni et al. [22] shifted the peak point toward the zero point by one unit and embedded the data in the peak and neighboring points. Note that the original peak point after embedding disappears in the histogram. Hence, to ensure ...
## 2.5. Lossless Compression–Based Schemes

### 2.5.1. Lossless Bit-Plane Compression in the Spatial Domain

Fridrich's group produced profound research on lossless data hiding techniques and developed a number of algorithms. This group proposed two techniques in this area [6]. The first technique, based on robust spatial additive watermarks, utilizes the modulo addition to embed the hash of the original image. The second technique uses the JBIG Lossless Compression Scheme [49] for losslessly compressing the bit-planes to make room for data embedding. To provide sufficient room for data embedding for the second technique, it is usual to compress the high level bit-plane. This mostly leads to visual quality degradation. Since the method aims at authentication, ...
## 2.6. Reversible Secret Sharing–Based Schemes

Secret sharing plays an important role in data encryption. Shamir et al. [56] proposed an (r, n)-threshold prototype based on Lagrange polynomial interpolation. In this model, the secret data are encrypted into n shares. If r or more than r shares are polled, the secret can be decrypted. Otherwise, if r−1 or fewer shares are collected, no meaningful information can be revealed. Thien et al. [57] extended Shamir et al.'s model into image secret sharing, i.e., hiding a secret image in a set of noiselike shadow images. Visual cryptography [58] is another useful technique for image secret sharing. It employs the properties of human visual system and thus maintains the advantage that the secret content ...
## 2.7. Summary

This chapter discussed the reversible data hiding schemes in the spatial domain. We first provide an overview of the spatial domain–based schemes. Then we discussed five types of spatial domain lossless information hiding methods, i.e., the modulo addition–based scheme, DE-based schemes, histogram modification–based schemes, lossless compression–based schemes, and reversible secret sharing–based schemes. With regard to DE-based schemes, we introduce two traditional DE-based schemes, i.e., Tian's scheme and Alatter's scheme.With regard to histogram modification–based schemes, we first introduce two traditional schemes, i.e., Ni et al.'s original scheme and Li et al.'s APD method. Then we introduce our reversible data hiding scheme ...
Chapter 3
# Lossless Information Hiding in Images on Transform Domains

## Abstract

As we know, lossless information hiding algorithms for images can be classified into three categories, i.e., the spatial domain–based, transform domain–based, and compressed domain–based schemes. The previous chapter focused on the spatial domain schemes; this chapter turns to transform domain–based schemes. We first introduce some related concepts and requirements for lossless information hiding. Then we give a brief overview of transform domain–based information hiding. Next we discuss some typical transform domain–based lossless information hiding methods, which are classified into two categories, i.e., integer discrete cosine transform (DCT)–based schemes and integer ...
## 3.2. Overview of Transform-Based Information Hiding

In general, transform-based information hiding algorithms are based on spectrum spread. Among various information hiding algorithms, transform domain–based algorithms are the most frequently used information-embedding algorithms. Compared with the spatial domain, transform domain is more complex, but it is robust, even if a lot of payload is embedded, the visual effect is still very good. Now a widely accepted view is that secret information should be embedded in the middle-frequency coefficients of the transform domain of the host image. This is because the high-frequency part has little effect on the quality of the original image, whereas the low-frequency part can achieve good robustness (after ...
## 3.3. Integer Discrete Cosine Transform–Based Schemes

### 3.3.1. Integer Discrete Cosine Transform

#### 3.3.1.1. One-Dimensional Integer Discrete Cosine Transform and Its Fast Algorithm

Let x(n) (n=0, 1, …, N−1) be a real input sequence. We assume that N=2t, where t>0. The scaled DCT of x(n) is defined as follows:X(k)=∑n=0N−1x(n)cosπ(2n+1)k2N,k=0,1,…,N−1

![image](/api/v2/epubs/urn:orm:book:9780128121665/files/IMAGES/B9780128120064000036/epub.stripin/si11.png) (3.10)

Let CN be the transform matrix of the DCT, that isCN={cosπ(2n+1)k2N}k,n=0,1,…,N−1

![image](/api/v2/epubs/urn:orm:book:9780128121665/files/IMAGES/B9780128120064000036/epub.stripin/si12.png) (3.11)

To derive the fast algorithm, we first get a factorization of the transform matrix based on the following lemma ...
## 3.4. Integer Wavelet Transform–Based Schemes

In this section, we turn to integer wavelet transform–based lossless information hiding schemes, which adopt the so-called CDF(2,2) wavelet. Thus, we first introduce related concept of the CDF(2,2) wavelet.The CDF(2,2) wavelet was proposed by Cohen, Daubechies, and Feauveau in 1992 [14]; it is a dual orthogonal (i.e., biorthogonal) (5,3) wavelet, whose low-pass and high-pass filters are with lengths of 5 and 3, respectively, both presenting symmetry. We can obtain an integer-to-integer bijective CDF(2,2) wavelet by lifting the original CDF (2,2) wavelet based on a certain lifting scheme, and this type of integer wavelet has been adopted in the JPEG2000 standard. Experiments have shown that, compared ...
## 3.5. Summary

This chapter has discussed transform domain lossless information hiding schemes, including IntDCT-based schemes and integer DWT-based schemes. Before introducing then, we introduced some related concepts and requirements for lossless information hiding, and gave a brief overview of transform domain–based information hiding.With regard to IntDCT-based schemes, we have introduced three Yang et al.'s schemes, i.e., the companding technique–based scheme, the histogram modification–based scheme, and the adaptive coefficient modification–based scheme. The first scheme takes advantage of the bit-shift operation in the companding process to achieve high watermarking capacity and good quality of watermarked images. To solve the over- and underflow ...
Chapter 4
# Lossless Information Hiding in Vector Quantization Compressed Images

## Abstract

The traditional pixel-domain information hiding algorithms may be greatly compromised when the images are encoded by lossy compression algorithms, which in this sense can be regarded as a type of attack to the original information hiding algorithms. This problem is especially urgent to the pixel-domain-based lossless information hiding algorithms because of the requirement of perfect restoration, which could be impaired by some lossy compression process. Nowadays, since more and more images are being stored in compressed formats such as JPEG and JPEG2000 or transmitted based on vector quantization (VQ) [[1]](../B9780128120064000048/B9780128120064000048_bib.xhtml#bib1) and block truncation coding (BTC) [[2]](../B9780128120064000048/B9780128120064000048_bib.xhtml#bib2), more and more ...
## 4.2. Overview of Vector Quantization–Based Information Hiding

Since 2000, VQ has been successfully applied to digital image watermarking [21–31]. Since 2005, because of the emergence of lossless information hiding, several scholars have been dedicated to lossless information hiding in VQ-compressed images. In the following two subsections, we overview typical methods related to VQ-based watermarking and VQ-based lossless information hiding.
### 4.2.1. Overview of Vector Quantization–Based Image Watermarking

Traditional digital watermarking schemes are mainly based on discrete cosine transform (DCT) and discrete wavelet transform. In the past 15years, many robust image watermarking techniques based on VQ [21–31] have been presented. These algorithms ...
## 4.3. Modified Fast Correlation Vector Quantization–Based Scheme

Image blocks encoded by VQ are usually in the form of indices, which represent codewords' positions in a codebook C. The VQ encoder E quantizes the input image block x by selecting a best-matched code vector cb [cb∈C={c0, c1,…, cN−1}, where N is the size of the codebook C]. Here, the Euclidean distance given in Eq. (4.2) is often employed as a metric for the best matching. It is obvious that this quantization might cause some distortion to the watermark information embedded in the pixel domain, if the embedding algorithms are sensitive to this operation, like most value expansion–based lossless information hiding algorithms [34–38]. So a lossless watermarking algorithm that works ...
## 4.4. Side Match Vector Quantization–Based Schemes

From Section 4.3, we can see that the hiding capacity of the MFCVQ method is unstable and low. To improve the hiding capacity, Chang et al. proposed several information hiding algorithms [4–7] based on SMVQ. We first introduce SMVQ in Section 4.4.1, and then introduce Chang and Wu's information hiding algorithm [4] in Section 4.4.2, Chang and Lu's lossless information hiding algorithm [5] in Section 4.4.3, and Chang-Tai-Lin's lossless information hiding scheme [6] in Section 4.4.4.
### 4.4.1. Side Match Vector Quantization

VQ takes advantage of the high degree of correlation between individual pixels within a block without considering the similarity between neighboring blocks. In VQ, each block is coded ...
## 4.5. Vector Quantization–Index Coding–Based Schemes

Achieving low image distortion (i.e., it reduces the suspicion of adversaries), high data embedding capacity, reversibility, and efficiency are desired by most information hiding system designers. Since no method is capable of achieving all these goals, a class of information hiding schemes is needed to span the range of possible applications. With the purpose of improving the visual quality of reconstructed images and achieving high embedding capacity and fast execution speed at the cost of a little higher bit rate, Chang et al., Wang et al., and Lu et al. propose a kind of lossless information embedding schemes for the steganographic application (i.e., secret communications) by using index ...
### 4.5.5. Improved Joint Neighboring Coding–Based Scheme

From the previous sections, we can see that MFCVQ, SMVQ, and JNC-based information hiding methods do not embed secret bits in the seed area. Furthermore, they perform the information hiding process in embeddable blocks one by one. In order to embed more secret bits, Lu et al.' presented an IJNC scheme [13]. In this scheme, the first strategy is to cancel the seed area and make all blocks embeddable. To reduce the coding bit rate, the second strategy is to perform the index coding process on each 2×2 index block rather than one by one. The main idea of the IJNC scheme is to first vector-quantize the cover image, obtaining an index table, and then divide the index table into nonoverlapping ...
## 4.6. Summary

This chapter discusses lossless information hiding schemes for VQ-compressed image. These schemes can be broadly classified into two categories, i.e., information hiding during VQ encoding and information hiding during VQ-index coding. The first category mainly includes MFCVQ-based and SMVQ-based schemes. The first category embeds information during VQ encoding, which can recover the VQ-compressed version after extraction of the embedded information. The second category views the index table as the cover object and embeds information during lossless index coding. In general, the schemes in the first category obtain worse stego images than index coding–based schemes. Furthermore, most of the index coding–based schemesnot only can further ...
Chapter 5
# Lossless Information Hiding in Block Truncation Coding–Compressed Images

## Abstract

This chapter focuses on block truncation coding (BTC)–based lossless information hiding schemes. BTC [[1]](../B978012812006400005X/B978012812006400005X_bib.xhtml#bib1) is a block-based spatial domain image compression technique for grayscale images. Its main idea is to quantize the pixels in each block into two levels while preserving certain statistical moments of small blocks of the grayscale image. The absolute moment BTC [[2]](../B978012812006400005X/B978012812006400005X_bib.xhtml#bib2) was proposed as a special kind of BTC to preserve the mean and the first absolute central moment of a block. After BTC compression, each block can be represented by a bitplane together with a pair of means called higher mean and lower mean. All higher means comprise a higher mean table, whereas ...
## 5.2. Overview of Block Truncation Coding–Based Information Hiding

In the 2000s, several watermarking and data hiding schemes for BTC-compressed gray-level images have been proposed. We overview them in two categories, i.e., watermarking schemes and lossless information hiding schemes.
### 5.2.1. Overview of Block Truncation Coding–Based Image Watermarking

The first watermarking method [3] based on BTC was proposed by Lu et al. in 2002, where the robust watermark is embedded by modifying the vector quantization (VQ)-BTC encoding process according to the watermark bits. In 2004, Lin and Chang [4] proposed a data hiding scheme for BTC-compressed images by performing least significant bit (LSB) substitution operations on BTC high and low means and performing ...
## 5.3. Bitplane Flipping–Based Lossless Hiding Schemes

In this section, we introduce the first reversible data hiding scheme for BTC-compressed gray-level images proposed by Hong et al. [9], and its improved scheme by Chen et al. [10]. Then, the reversible data hiding scheme [13] for BTC-compressed images based on bitplane flipping together with histogram shifting of mean tables is introduced, which is proposed by the author of this book.Here, the original AMBTC method is used in all these methods, assuming that the M×N-sized 256-gray-level image X is divided into nonoverlapping m×n-sized blocks, i.e., X={x(ij), 1≤i≤M/m, 1≤j≤N/n}. The pixels in each block are individually quantized into two-level outputs in such a way that the mean ...
## 5.4. Mean Coding–Based Lossless Hiding Schemes

The aforementioned data hiding schemes in the BTC domain modify either the BTC encoding stage or the BTC-compressed data according to the secret bits, and they have no ability to reduce the bit rate but may reduce the image quality. To reduce the bit rate and increase the hiding capacity, this section introduces our three reversible data hiding schemes for BTC-compressed images, which can further losslessly encode the BTC-compressed data according to secret data.
### 5.4.1. Prediction-Error Expansion–Based Scheme

In 2012, we proposed a reversible data hiding scheme for BTC-compressed images by introducing the prediction-error expansion technique [15]. For the purpose of increasing the capacity, all high ...
## 5.5. Lossless Data Hiding in Block Truncation Coding–Compressed Color Images

As we know, BTC [1] is a lossy compression technique that can significantly reduce the size of digital images with acceptable visual quality. The traditional color BTC method compresses each color image block (typically 4×4) into three high means, three low means, and three bitplanes. To conceal secret data into color BTC compression codes as well as reducing the number of bitplanes, Chang et al. [5] used the genetic algorithm (GA) to generate an optimal common bitplane (replacing the traditional three bitplanes) to reduce the bit rate, and then utilized the side match distortion concept to increase the embedding capacity. However, Chang et al.'s method is time consuming ...
## 5.6. Summary

This chapter discusses lossless information hiding schemes for BTC-compressed image. These schemes can be broadly classified into three categories, i.e., bitplane flipping–based schemes for BTC-compressed grayscale images, mean coding–based schemes for BTC-compressed grayscale images, and lossless data hiding schemes in BTC-compressed color images.The first category includes three schemes, i.e., original bitplane flipping–based scheme, improved bitplane flipping–based scheme, and bitplane flipping–based scheme with histogram shifting of mean tables. The third method is proposed by us. Through histogram shifting of mean tables, our scheme can maintain the same PSNR values as the original AMBTC technique. We embed secret bits in two ...
Chapter 6
# Lossless Information Hiding in JPEG- and JPEG2000-Compressed Images

## Abstract

As we know, most existing multimedia data are stored in compressed formats, thus the data hiding techniques for compressed images become much more practical. [Chapters 4](../B9780128120064000048/B9780128120064000048.xhtml#c0004) and [5](../B978012812006400005X/B978012812006400005X.xhtml#c0005) focused on lossless information hiding techniques for vector quantization–and block truncation coding–compressed images, respectively; this chapter deals with lossless information hiding techniques for JPEG- and JPEG2000-compressed images in this chapter. The JPEG format is the most popular image format in current use. This is because it offers a convenient trade-off between the file size and the perceptual quality of the encoded image. Thus the lossless information hiding schemes for JPEG ...
## 6.2. Lossless Information Hiding in JPEG Images

### 6.2.1. Overview of Information Hiding in JPEG Images

The JPEG image format is a widely used compressed format. For JPEG images, as early as in 1997, Upham [7] first developed a famous hiding tool for JPEG images named Jpeg-Jsteg, where the secret data are embedded into the least significant bits (LSBs) of the quantized DCT coefficients whose values are not 0, 1 or −1. Westfeld [8] developed the so-called F5 algorithm, which implements matrix encoding to improve the embedding efficiency. In addition, F5 also employs permutative straddling to uniformly spread out the changes over the whole steganogram. Both these methods are irreversible with a low capacity. In the same year, Fridrich et al. [9] presented ...
## 6.3. Lossless Information Hiding in JPEG2000 Images

As an important branch of information security, information hiding in images has been extensively studied in recent years. However, most schemes will introduce irreversible distortion to the original image. It is unallowable in some special applications such as legal imaging and medical imaging. So reversible information hiding deserves us to study. In addition, most existing information hiding schemes are for BMP and JPEG images. There are few related schemes for JPEG2000 images. So it is very necessary for us to study reversible information hiding for JPEG2000 images.
### 6.3.1. Overview

To understand the following schemes better, we introduce the JPEG2000 codec first. The block diagram of the JPEG2000 ...
## 6.4. Summary

This chapter focuses on the lossless information hiding in JPEG and JPEG2000 images.For JPEG images, seven typical methods including two schemes from the authors were introduced. For the first five schemes, we introduced the basic ideas, whereas for the schemes from the authors, we provided a detailed introduced detail with experimental results. In the first method of ours, our high-capacity reversible JPEG-to-JPEG data hiding scheme is introduced. Through lowering certain quantization table entries and lifting corresponding quantized DCT coefficients, space is made for embedding data. Using the proposed embedding strategy, our scheme can achieve high embedding capacity and keep the distortion introduced by embedding very low; meanwhile ...
