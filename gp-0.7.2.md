# JAM: JOIN-ACCUMULATE MACHINE

## A MOSTLY-COHERENT TRUSTLESS SUPERCOMPUTER

**DRAFT 0.6.6 - May 5, 2025**

**DR. GAVIN WOOD**
FOUNDER, POLKADOT & ETHEREUM
GAVIN@PARITY.IO

---

## Abstract

We present a comprehensive and formal definition of Jam, a protocol combining elements of both Polkadot and Ethereum. In a single coherent model, Jam provides a global singleton permissionless object environment—much like the smart-contract environment pioneered by Ethereum—paired with secure sideband computation parallelized over a scalable node network, a proposition pioneered by Polkadot. Jam introduces a decentralized hybrid system offering smart-contract functionality structured around a secure and scalable in-core/on-chain dualism. While the smart-contract functionality implies some similarities with Ethereum's paradigm, the overall model of the service offered is driven largely by underlying architecture of Polkadot. Jam is permissionless in nature, allowing anyone to deploy code as a service on it for a fee commensurate with the resources this code utilizes and to induce execution of this code through the procurement and allocation of core-time, a metric of resilient and ubiquitous computation, somewhat similar to the purchasing of gas in Ethereum. We already envision a Polkadot-compatible CoreChains service.

---

## 1. Introduction

### 1.1. Nomenclature

In this paper, we introduce a decentralized, crypto-economic protocol to which the Polkadot Network will transition itself in a major revision on the basis of approval by its governance apparatus. An early, unrefined, version of this protocol was first proposed in Polkadot Fellowship rfc 31, known as CoreJam. CoreJam takes its name after the collect/refine/join/accumulate model of computation at the heart of its service proposition. While the CoreJam rfc suggested an incomplete, scope-limited alteration to the Polkadot protocol, Jam refers to a complete and coherent overall blockchain protocol.

### 1.2. Driving Factors

Within the realm of blockchain and the wider Web3, we are driven by the need first and foremost to deliver resilience. A proper Web3 digital system should honor a declared service profile—and ideally meet even perceived expectations—regardless of the desires, wealth or power of any economic actors including individuals, organizations and, indeed, other Web3 systems. Inevitably this is aspirational, and we must be pragmatic over how perfectly this may really be delivered. Nonetheless, a Web3 system should aim to provide such radically strong guarantees that, for practical purposes, the system may be described as unstoppable. While Bitcoin is, perhaps, the first example of such a system within the economic domain, it was not general purpose in terms of the nature of the service it offered. A rules-based service is only as useful as the generality of the rules which may be conceived and placed within it. Bitcoin's rules allowed for an initial use-case, namely a fixed-issuance token, ownership of which is well-approximated and autonomously enforced through knowledge of a secret, as well as some further elaborations on this theme. Later, Ethereum would provide a categorically more general-purpose rule set, one which was practically Turing complete. In the context of Web3 where we are aiming to deliver a massively multiuser application platform, generality is crucial, and thus we take this as a given. Beyond resilience and generality, things get more interesting, and we must look a little deeper to understand what our driving factors are. For the present purposes, we identify three additional goals: (1) Resilience: highly resistant from being stopped, corrupted and censored. (2) Generality: able to perform Turing-complete computation. The gas mechanism did restrict what programs can execute on it by placing an upper bound on the number of steps which may be executed, but some restriction to avoid infinite-computation must surely be introduced in...

### 1.3. Scaling under Size-Coherency Antagonism

Size-coherency antagonism is a simple principle implying that as the state-space of information systems grow, then the system necessarily becomes less coherent. It is a direct implication of principle that causality is limited by speed. The maximum speed allowed by physics is C the speed of light in a vacuum, however other information systems may have lower bounds: In biological system this is largely determined by various chemical processes whereas in electronic systems is it determined by the speed of electrons in various substances. Distributed software systems will tend to have much lower bounds still, being dependent on a substrate of software, hardware and packet-switched networks of varying reliability. The argument goes: (1) The more state a system utilizes for its data-processing, the greater the amount of space this state must occupy. (2) The more space used, then the greater the mean and variance of distances between state-components. (3) As the mean and variance increase, then time for causal resolution (i.e. all correct implications of an event to be felt) becomes divergent across the system, causing incoherence. Setting the question of overall security aside for a moment, we can manage incoherence by fragmenting the system into causally-independent subsystems, each of which is small enough to be coherent. In a resource-rich environment, a bacterium may split into two rather than growing to double its size. This pattern is rather a crude means of dealing with incoherency under growth: intra-system processing has low size and total coherence, inter-system processing supports higher overall sizes but without coherence. It is the principle behind meta-networks such as Polkadot, Cosmos and the predominant vision of a scaled Ethereum (all to be discussed in depth shortly). Such systems typically rely on asynchronous and simplistic communication with "settlement areas" which provide a small-scoped coherent state-spa...

### 1.4. Document Structure

We begin with a brief overview of present scaling approaches in blockchain technology in section 2. In section 3 we define and clarify the notation from which we will draw for our formalisms. We follow with a broad overview of the protocol in section 4 outlining the major areas including the Polkadot Virtual Machine (pvm), the consensus protocols Safrole and Grandpa, the common clock and build the foundations of the formalism. We then continue with the full protocol definition split into two parts: firstly the correct on-chain state-transition formula helpful for all nodes wishing to validate the chain state, and secondly, in sections 14 and 19 the honest strategy for the off-chain actions of any actors who wield a validator key. The main body ends with a discussion over the performance characteristics of the protocol in section 20 and finally conclude in section 21. The appendix contains various additional material important for the protocol definition including the pvm in appendices A & B, serialization and Merklization in appendices C & D and cryptography in appendices E, G & H. We finish with an index of terms which includes the values of all simple constant terms used in the work in appendix I, and close with the bibliography.

---

## 2. Previous Work and Present Trends

In the years since the initial publication of the Ethereum YP, the field of blockchain development has grown immensely. Other than scalability, development has been done around underlying consensus algorithms, smart-contract languages and machines and overall state environments. While interesting, these latter subjects are mostly out scope of the present work since they generally do not impact underlying scalability.

### 2.1. Polkadot

In order to deliver its service, Jam coopts much of the same game-theoretic and cryptographic machinery as Polkadot known as Elves and described by Jeff Burdges, Cevallos, et al. 2024. However, major differences exist in the actual service offered with Jam, providing an abstraction much closer to the actual computation model generated by the validator nodes its economy incentivizes. It was a major point of the original Polkadot proposal, a scalable heterogeneous multichain, to deliver high-performance through partition and distribution of the workload over multiple host machines. In doing so it took an explicit position that composability would be lowered. Polkadot's constituent components, parachains are, practically speaking, highly isolated in their nature. Though a message passing system (xcmp) exists it is asynchronous, coarse-grained and practically limited by its reliance on a high-level slowly evolving interaction language xcm. As such, the composability offered by Polkadot between its constituent chains is lower than that of Ethereum-like smart-contract systems offering a single and universal object environment and allowing for the kind of agile and innovative integration which underpins their success. Polkadot, as it stands, is a collection of independent ecosystems with only limited opportunity for collaboration, very similar in ergonomics to bridged blockchains though with a categorically different security profile. A technical proposal known as spree would utilize Polkadot's unique shared-security and improve composability, though blockchains would still remain isolated. Implementing and launching a blockchain is hard, time-consuming and costly. By its original design, Polkadot limits the clients able to utilize its service to those who are both able to do this and raise a sufficient deposit to win an auction for a long-term slot, one of around 50 at the present time. While not perm...

### 2.2. Ethereum

The Ethereum protocol was formally defined in this paper's spiritual predecessor, the Yellow Paper, by Wood 2014. This was derived in large part from the initial concept paper by Buterin 2013. In the decade since the YP was published, the de facto Ethereum protocol and public network instance have gone through a number of evolutions, primarily structured around introducing flexibility via the transaction format and the instruction set and "precompiles" (niche, sophisticated bonus instructions) of its scripting core, the Ethereum virtual machine (evm). Almost one million crypto-economic actors take part in the validation for Ethereum. Block extension is done through a randomized leader-rotation method where the physical address of the leader is public in advance of their block production. Ethereum uses Casper-FFG introduced by Buterin and Griffith 2019 to determine finality, which with the large validator base finalizes the chain extension around every 13 minutes. Ethereum's direct computational performance remains broadly similar to that with which it launched in 2015, with a notable exception that an additional service now allows 1 mb of commitment data to be hosted per block (all nodes to store it for a limited period). The data cannot be directly utilized by the main state-transition function, but special functions provide proof that the data (or some subsection thereof) is available. According to Ethereum Foundation 2024b, the present design direction is to improve on this over the coming years by splitting responsibility for its storage amongst the validator base in a protocol known as Dank-sharding. According to Ethereum Foundation 2024a, the scaling strategy of Ethereum would be to couple this data availability with a private market of roll-ups, sideband computation facilities of various design, with zk-snark-based roll-ups being a stated preference. Each vendor's roll-up design, execution and operation comes with its own implications. On...

### 2.3. Fragmented Meta-Networks

Directions for general-purpose computation scalability taken by other projects broadly centre around one of two approaches; either what might be termed a fragmentation approach or alternatively a centralization approach. We argue that neither approach offers a compelling solution. The fragmentation approach is heralded by projects such as Cosmos (proposed by Kwon and Buchman 2019) and Avalanche (by Tanana 2019). It involves a system fragmented by networks of a homogenous consensus mechanic, yet staffed by separately motivated sets of validators. This is in contrast to Polkadot's single validator set and Ethereum's declared strategy of heterogeneous rollups secured partially by the same validator set operating under a coherent incentive framework. The homogeneity of said fragmentation approach allows for reasonably consistent messaging mechanics, helping to present a fairly unified interface to the multitude of connected networks. However, the apparent consistency is superficial. The networks are trustless only by assuming correct operation of their validators, who operate under a crypto-economic security framework ultimately conjured and enforced by economic incentives and punishments. To do twice as much work with the same levels of security and no special coordination between validator sets, then such systems essentially prescribe forming a new network with the same overall levels of incentivization. Several problems arise. Firstly, there is a similar downside as with Polkadot's isolated parachains and Ethereum's isolated roll-up chains: a lack of coherency due to a persistently sharded state preventing synchronous composability. More problematically, the scaling-by-fragmentation approach, proposed specifically by Cosmos, provides no homogenous security—and therefore trustlessness—guarantees. Validator sets between networks must be assumed to be independently selected and incentivized with no relationship, causal or probabilistic,...

### 2.4. High-Performance Fully Synchronous Networks

Another trend in the recent years of blockchain development has been to make "tactical" optimizations over data throughput by limiting the validator set size or diversity, focusing on software optimizations, requiring a higher degree of coherency between validators, onerous requirements on the hardware which validators must have, or limiting data availability. The Solana blockchain is underpinned by technology introduced by Yakovenko 2018 and boasts theoretical figures of over 700,000 transactions per second, though according to Ng 2024 the network is only seen processing a small fraction of this. The underlying throughput is still substantially more than most blockchain networks and is owed to various engineering optimizations in favor of maximizing synchronous performance. The result is a highly-coherent smart-contract environment with an api not unlike that of YP Ethereum (albeit using a different underlying vm), but with a near-instant time to inclusion and finality which is taken to be immediate upon inclusion. Two issues arise with such an approach: firstly, defining the protocol as the outcome of a heavily optimized codebase creates structural centralization and can undermine resilience. Jha 2024 writes "since January 2022, 11 significant outages gave rise to 15 days in which major or partial outages were experienced". This is an outlier within the major blockchains as the vast majority of major chains have no downtime. There are various causes to this downtime, but they are generally due to bugs found in various subsystems. Ethereum, at least until recently, provided the most contrasting alternative with its well-reviewed specification, clear research over its crypto-economic foundations and multiple clean-room implementations. It is perhaps no surprise that the network very notably continued largely unabated when a flaw in its most deployed implementation was found and maliciously exploited, as described by...

---

## 3. Notational Conventions

Much as in the Ethereum Yellow Paper, a number of notational conventions are used throughout the present work. We define them here for clarity. The Ethereum Yellow Paper itself may be referred to henceforth as the YP.

### 3.1. Typography

We use a number of different typefaces to denote different kinds of terms. Where a term is used to refer to a value only relevant within some localized section of the document, we use a lower-case roman letter e.g. x, y (typically used for an item of a set or sequence) or e.g. i, j (typically used for numerical indices). Where we refer to a Boolean term or a function in a local context, we tend to use a capitalized roman alphabet letter such as A, F. If particular emphasis is needed on the fact a term is sophisticated or multidimensional, then we may use a bold typeface, especially in the case of sequences and sets. For items which retain their definition throughout the present work, we use other typographic conventions. Sets are usually referred to with a blackboard typeface, e.g. N refers to all natural numbers including zero. Sets which may be parameterized may be subscripted or be followed by parenthesized arguments. Imported functions, used by the present work but not specifically introduced by it, are written in calligraphic typeface, e.g. H the Blake2 cryptographic hashing function. For other non-context dependent functions introduced in the present work, we use upper case Greek letters, e.g. Υ denotes the state transition function. Values which are not fixed but nonetheless hold some consistent meaning throughout the present work are denoted with lower case Greek letters such as σ, the state identifier. These may be placed in bold typeface to denote that they refer to an abnormally complex value.

### 3.2. Functions and Operators

We define the precedes relation to indicate that one term is defined in terms of another. E.g. y ≺ x indicates that y may be defined purely in terms of x : y ≺ x ⇐⇒ ∃ f ∶ y = f (x) (3.1) The substitute-if-nothing function U is equivalent to the first argument which is not ∅, or ∅ if no such argument exists: U (a 0 ,. .. a n) ≡ a x ∶ (a x ≠ ∅ ∨ x = n), x − 1 ⋀ i = 0 a i = ∅ (3.2) Thus, e.g. U (∅, 1, ∅, 2) = 1 and U (∅, ∅) = ∅.

### 3.3. Sets

Given some set s, its power set and cardinality are denoted as the usual ℘ ⟨ s ⟩ and S s S. When forming a power set, we may use a numeric subscript in order to restrict the resultant expansion to a particular cardinality. E.g. ℘ ⟨{ 1, 2, 3 }⟩ 2 = {{ 1, 2 }, { 1, 3 }, { 2, 3 }}. Sets may be operated on with scalars, in which case the result is a set with the operation applied to each element, e.g. { 1, 2, 3 } + 3 = { 4, 5, 6 }. Functions may also be applied to all members of a set to yield a new set, but for clarity we denote this with a # superscript, e.g. f # ({ 1, 2 }) ≡ { f (1), f (2)}. We denote set-disjointness with the relation ⫰. Formally: A ∩ B = ∅ ⇐⇒ A ⫰ B We commonly use ∅ to indicate that some term is validly left without a specific value. Its cardinality is defined as zero. We define the operation ? such that A ? ≡ A ∪ { ∅ } indicating the same set but with the addition of the ∅ element. The term ∇ is utilized to indicate the unexpected failure of an operation or that a value is invalid or unexpected. (We try to avoid the use of the more conventional ⊥ here to avoid confusion with Boolean false, which may be interpreted as some successful result in some contexts.)

### 3.4. Numbers

N denotes the set of naturals including zero whereas N n implies a restriction on that set to values less than n. Formally, N = { 0, 1 ,. .. } and N n = { x S x ∈ N, x < n }. Z denotes the set of integers. We denote Z a...b to be the set of integers within the interval [ a, b). Formally, Z a...b = { x S x ∈ Z, a ≤ x < b }. E.g. Z 2 ... 5 = { 2, 3, 4 }. We denote the offset/length form of this set as Z a ⋅⋅⋅+ b, a short form of Z a...a + b. It can sometimes be useful to represent lengths of sequences and yet limit their size, especially when dealing with sequences of octets which must be stored practically. Typically, these lengths can be defined as the set N 2 32. To improve clarity, we denote N L as the set of lengths of octet sequences and is equivalent to N 2 32. We denote the % operator as the modulo operator, e.g. 5 % 3 = 2. Furthermore, we may occasionally express a division result as a quotient and remainder with the separator R, e.g. 5 ÷ 3 = 1 R 2.

### 3.5. Dictionaries

A dictionary is a possibly partial mapping from some domain into some co-domain in much the same manner as a regular function. Unlike functions however, with dictionaries the total set of pairings are necessarily enumerable, and we represent them in some data structure as the set of all (key ↦ value) pairs. (In such data-defined mappings, it is common to name the values within the domain a key and the values within the co-domain a value, hence the naming.) Thus, we define the formalism D ⟨ K → V ⟩ to denote a dictionary which maps from the domain K to the range V. We define a dictionary as a member of the set of all dictionaries D and a set of pairs p = (k ↦ v) : D ⊂ {(k ↦ v)} (3.3) A dictionary's members must associate at most one unique value for any key k : ∀ d ∈ D ∶ ∀ (k ↦ v) ∈ d ∶ ∃ ! v ′ ∶ (k ↦ v ′) ∈ d (3.4) This assertion allows us to unambiguously define the subscript and subtraction operator for a dictionary d : ∀ d ∈ D ∶ d [ k ] ≡ { v if ∃ k ∶ (k ↦ v) ∈ d, ∅ otherwise } (3.5) ∀ d ∈ D, s ⊆ K ∶ d ∖ s ≡ {(k ↦ v) ∶ (k ↦ v) ∈ d, k ∉ s } (3.6) Note that when using a subscript, it is an implicit assertion that the key exists in the dictionary. Should the key not exist, the result is undefined and any block which relies on it must be considered invalid. It is typically useful to limit the sets from which the keys and values may be drawn. Formally, we define a typed dictionary D ⟨ K → V ⟩ as a set of pairs p of the form (k ↦ v) : D ⟨ K → V ⟩ ⊂ D (3.7) D ⟨ K → V ⟩ ≡ {(k ↦ v) S k ∈ K ∧ v ∈ V } (3.8) To denote the active domain (i.e. set of keys) of a dictionary d ∈ D ⟨ K → V ⟩, we use K (d) ⊆ K and for the range (i.e. set of values), V (d) ⊆ V. Formally: K (d ∈ D) ≡ { k S ∃ v ∶ (k ↦ v) ∈ d } (3.9) V (d ∈ D) ≡ { v S ∃ k ∶ (k ↦ v) ∈ d } (3.10) Note that since the co-domain of V is a set, should different keys with equal values appear in the dictionary, the set will only contain one such value. Dictionaries may be combined through th...

### 3.6. Tuples

Tuples are groups of values where each item may belong to a different set. They are denoted with parentheses, e.g. the tuple t of the naturals 3 and 5 is denoted t = (3, 5), and it exists in the set of natural pairs sometimes denoted N × N, but denoted in the present work as (N, N). We have frequent need to refer to a specific item within a tuple value and as such find it convenient to declare a name for each item. E.g. we may denote a tuple with two named natural components a and b as T = { a ∈ N, b ∈ N }. We would denote an item t ∈ T through subscripting its name, thus for some t = { a ▸ ▸ 3, b ▸ ▸ 5 }, t a = 3 and t b = 5.

### 3.7. Sequences

A sequence is a series of elements with particular ordering not dependent on their values. The set of sequences of elements all of which are drawn from some set T is denoted ⟦ T ⟧, and it defines a partial mapping N → T. The set of sequences containing exactly n elements each a member of the set T may be denoted ⟦ T ⟧ n and accordingly defines a complete mapping N n → T. Similarly, sets of sequences of at most n elements and at least n elements may be denoted ⟦ T ⟧ ∶ n and ⟦ T ⟧ n ∶ respectively. Sequences are subscriptable, thus a specific item at index i within a sequence s may be denoted s [ i ], or where unambiguous, s i. A range may be denoted using an ellipsis for example: [ 0, 1, 2, 3 ] ... 2 = [ 0, 1 ] and [ 0, 1, 2, 3 ] 1 ⋅⋅⋅+ 2 = [ 1, 2 ]. The length of such a sequence may be denoted S s S. We denote modulo subscription as s [ i ] ↺ ≡ s [ i % S s S ]. We denote the final element x of a sequence s = [ ..., x ] through the function last (s) ≡ x.

#### 3.7.1. Construction

We may wish to define a sequence in terms of incremental subscripts of other values: [ x 0, x 1 ,. .. ] n denotes a sequence of n values beginning x 0 continuing up to x n − 1. Furthermore, we may also wish to define a sequence as elements each of which are a function of their index i ; in this case we denote [ f (i) S i ← N n ] ≡ [ f (0), f (1) ,. .., f (n − 1)]. Thus, when the ordering of elements matters we use ← rather than the unordered notation ∈. The latter may also be written in short form [ f (i ← N n)]. This applies to any set which has an unambiguous ordering, particularly sequences, thus [ i 2 S i ← [ 1, 2, 3 ] ] = [ 1, 4, 9 ]. Multiple sequences may be combined, thus [ i ⋅ j S i ← [ 1, 2, 3 ], j ← [ 2, 3, 4 ] ] = [ 2, 6, 12 ]. As with sets, we use explicit notation f # to denote a function mapping over all items of a sequence. Sequences may be constructed from sets or other sequences whose order should be ignored through sequence ordering notation [ i k ...

### 3.8. Cryptography

#### 3.8.1. Hashing

H denotes the set of 256-bit values typically expected to be arrived at through a cryptographic function, equivalent to Y 32, with H 0 being equal to [ 0 ] 32. We assume a function H (m ∈ Y) ∈ H denoting the Blake2b 256-bit hash introduced by Saarinen and Aumasson 2015 and a function H K (m ∈ Y) ∈ H denoting the Keccak 256-bit hash as proposed by Bertoni et al. 2013 and utilized by Wood 2014. We may sometimes wish to take only the first x octets of a hash, in which case we denote H x (m) ∈ Y x to be the first x octets of H (m). The inputs of a hash function should be expected to be passed through our serialization codec E to yield an octet sequence to which the cryptography may be applied. (Note that an octet sequence conveniently yields an identity transform.) We may wish to interpret a sequence of octets as some other kind of value with the assumed decoder function E − 1 (x ∈ Y). In both cases, we may subscript the transformation function with the number of octets we expect the octet sequence term to have. Thus, r = E 4 (x ∈ N) would assert x ∈ N 2 32 and r ∈ Y 4, whereas s = E − 1 8 (y) would assert y ∈ Y 8 and s ∈ N 2 64.

#### 3.8.2. Signing Schemes

E k ⟨ m ⟩ ⊂ Y 64 is the set of valid Ed25519 signatures, defined by Josefsson and Liusvaara 2017, made through knowledge of a secret key whose public key counterpart is k ∈ Y 32 and whose message is m. To aid readability, we denote the set of valid public keys H E. We use Y BLS ⊂ Y 144 to denote the set of public keys for the bls signature scheme, described by Boneh, Lynn, and Shacham 2004, on curve bls 12 - 381 defined by Hopwood et al. 2020. We denote the set of valid Bandersnatch public keys as H B, defined in appendix G. F m ∈ Y k ∈ H B ⟨ x ∈ Y ⟩ ⊂ Y 96 is the set of valid singly-contextualized signatures of utilizing the secret counterpart to the public key k, some context x and message m. F̄ m ∈ Y r ∈ Y R ⟨ x ∈ Y ⟩ ⊂ Y 784...

---

## 4. Overview

As in the Yellow Paper, we begin our formalisms by recalling that a blockchain may be defined as a pairing of some initial state together with a block-level state-transition function. The latter defines the posterior state given a pairing of some prior state and a block of data applied to it. Formally, we say: σ ′ ≡ Υ (σ, B) (4.1) Where σ is the prior state, σ ′ is the posterior state, B is some valid block and Υ is our block-level state-transition function. Broadly speaking, Jam (and indeed blockchains in general) may be defined simply by specifying Υ and some genesis state σ 0. We also make several additional assumptions of agreed knowledge: a universally known clock, and the practical means of sharing data with other systems operating under the same consensus rules. The latter two were both assumptions silently made in the YP.

### 4.1. The Block

To aid comprehension and definition of our protocol, we partition as many of our terms as possible into their functional components. We begin with the block B which may be restated as the header H and some input data external to the system and thus said to be extrinsic, E : B ≡ (H, E) (4.2) E ≡ (E T, E D, E P, E A, E G) (4.3) The header is a collection of metadata primarily concerned with cryptographic references to the blockchain ancestors and the operands and result of the present transition. As an immutable known a priori, it is assumed to be available throughout the functional components of block transition. The extrinsic data is split into its several portions: tickets: Tickets, used for the mechanism which manages the selection of validators for the permissioning of block authoring. This component is denoted E T. preimages: Static data which is presently being requested to be available for workloads to be able to fetch on demand. This is denoted E P. reports: Reports of newly completed workloads whose accuracy is guaranteed by specific validators. This is denoted E G. availability: Assurances by each validator concerning which of the input data of workloads they have correctly received and are storing locally. This is denoted E A. disputes: Information relating to disputes between validators over the validity of reports. This is denoted E D.

### 4.2. The State

Our state may be logically partitioned into several largely independent segments which can both help avoid visual clutter within our protocol description and provide formality over elements of computation which may be simultaneously calculated (i.e. parallelized). We therefore pronounce an equivalence between σ (some complete state) and a tuple of partitioned segments of that state: σ ≡ (α, β, γ, δ, η, ι, κ, λ, ρ, τ, φ, χ, ψ, π, ϑ, ξ) (4.4) In summary, δ is the portion of state dealing with services, analogous in Jam to the Yellow Paper's (smart contract) accounts, the only state of the YP's Ethereum. The identities of services which hold some privileged status are tracked in χ. Validators, who are the set of economic actors uniquely privileged to help build and maintain the Jam chain, are identified within κ, archived in λ and enqueued from ι. All other state concerning the determination of these keys is held within γ. Note this is a departure from the YP proof-of-work definitions which were mostly stateless, and this set was not enumerated but rather limited to those with sufficient compute power to find a partial hash-collision in the sha 2 - 256 cryptographic hash function. An on-chain entropy pool is retained in η. Our state also tracks two aspects of each core: α, the authorization requirement which work done on that core must satisfy at the time of being reported on-chain, together with the queue which fills this, φ ; and ρ, each of the cores' currently assigned report, the availability of whose work-package must yet be assured by a super-majority of validators. Finally, details...

### 4.3. Which History?

A blockchain is a sequence of blocks, each cryptographically referencing some prior block by including a hash of its header, all the way back to some first block which references the genesis header. We already presume consensus over this genesis header H 0 and the state it represents already defined as σ 0. By defining a deterministic function for deriving a single posterior state for any (valid) combination of prior state and block, we are able to define a unique canonical state for any given block. We generally call the block with the most ancestors the head and its state the head state. It is generally possible for two blocks to be valid and yet reference the same prior block in what is known as a fork. This implies the possibility of two different heads, each with their own state. While we know of no way to strictly preclude this possibility, for the system to be useful we must nonetheless attempt to minimize it. We therefore strive to ensure that: (1) It be generally unlikely for two heads to form. (2) When two heads do form they be quickly resolved into a single head. (3) It be possible to identify a block not much older than the head which we can be extremely confident will form part of the blockchain's history in perpetuity. When a block becomes identified as such we call it finalized and this property naturally extends to all of its ancestor blocks. These goals are achieved through a combination of two consensus mechanisms: Safrole, which governs the (not-necessarily forkless) extension of the blockchain; and Grandpa, which governs the finalization of some extension into canonical history. Thus, the former delivers point 1, the latter delivers point 3 and both are important for delivering point 2. We describe these portions of the protocol in detail in sections 6 and 19 respectively. While Safrole limits forks to a large extent (through cryptography, economics and common-time, below), there may be times when we wish to intentionally f...

### 4.4. Time

We presume a pre-existing consensus over time specifically for block production and import. While this was not an assumption of Polkadot, pragmatic and resilient solutions exist including the ntp protocol and network. We utilize this assumption in only one way: we require that blocks be considered temporarily invalid if their timeslot is in the future. This is specified in detail in section 6. Formally, we define the time in terms of seconds passed since the beginning of the Jam Common Era, 1200 UTC on January 1, 2025. Midday UTC is selected to ensure that all major timezones are on the same date at any exact 24-hour multiple from the beginning of the common era. Formally, this value is denoted T.

### 4.5. Best block

Given the recognition of a number of valid blocks, it is necessary to determine which should be treated as the "best" block, by which we mean the most recent block we believe will ultimately be within of all future Jam chains. The simplest and least risky means of doing this would be to inspect the Grandpa finality mechanism which is able to provide a block for which there is a very high degree of confidence it will remain an ancestor to any future chain head. However, in reducing the risk of the resulting block ultimately not being within the canonical chain, Grandpa will typically return a block some small period older than the most recently authored block. (Existing deployments suggest around 1-2 blocks in the past under regular operation.) There are often circumstances when we may wish to have less latency at the risk of the returned block not ultimately forming a part of the future canonical chain. E.g. we may be in a position of being able to author a block, and we need to decide what its parent should be. Alternatively, we may care to speculate about the most recent state for the purpose of providing information to a downstream application reliant on the state of Jam. In these cases, we define the best block as the head of the best chain, itself defined in section 19.

### 4.6. Economics

The present work describes a cryptoeconomic system, i.e. one combining elements of both cryptography and economics and game theory to deliver a self-sovereign digital service. In order to codify and manipulate economic incentives we define a token which is native to the system, which we will simply call tokens in the present work. A value of tokens is generally referred to as a balance, and such a value is said to be a member of the set of balances, N B, which is exactly equivalent to the set of naturals less than 2 64 (i.e. 64-bit unsigned integers in coding parlance). Formally: N B ≡ N 2 64 (4.21) Though unimportant for the present work, we presume that there be a standard named denomination for 10 9 tokens. This is different to both Ethereum (which uses a denomination of 10 18), Polkadot (which uses a denomination of 10 10) and Polkadot's experimental cousin Kusama (which uses 10 12). The fact that balances are constrained to being less than 2 64 implies that there may never be more than around 18 × 10 9 tokens (each divisible into portions of 10 − 9) within Jam. We would expect that the total number of tokens ever issued will be a substantially smaller amount than this. We further presume that a number of constant prices stated in terms of tokens are known. However we leave the specific values to be determined in following work: B I : the additional minimum balance implied for a single item within a mapping. B L : the additional minimum balance implied for a single octet of data within a mapping. B S : the minimum balance implied for a service.

### 4.7. The Virtual Machine and Gas

In the present work, we presume the definition of a Polkadot Virtual Machine (pvm). This virtual machine is based around the risc-v instruction set architecture, specifically the rv 64 em variant, and is the basis for introducing permissionless logic into our state-transition function. The pvm is comparable to the evm defined in the Yellow Paper, but somewhat simpler: the complex instructions for cryptographic operations are missing as are those which deal with environmental interactions. Overall it is far less opinionated since it alters a pre-existing general purpose design, risc-v, and optimizes it for our needs. This gives us excellent pre-existing tooling, since pvm remains essentially compatible with risc-v, including support from the compiler toolkit llvm and languages such as Rust and C++. Furthermore, the instruction set simplicity which risc-v and pvm share, together with the register size (64-bit), active number (13) and endianness (little) make it especially well-suited for creating efficient recompilers on to common hardware architectures. The pvm is fully defined in appendix A, but for contextualization we will briefly summarize the basic invocation function Ψ which computes the resultant state of a pvm instance initialized with some registers (⟦ N R ⟧ 13) and ram (M) and has executed for up to some amount of gas (N G), a number of approximately time-proportional computational steps: (4.22) Ψ ∶ { Y, N R, N G, ⟦ N R ⟧ 13, M } → { { ∎, ☇, ∞ } ∪ { F, h̵ } × N R, N R, Z G, ⟦ N R ⟧ 13, M } We refer to the time-proportional computational steps as gas (much like in the YP) and limit it to a 64-bit quantity. We may use either N G or Z G to bound it, the first as a prior argument since it is known to be positive, the latter as a result where a negative value indicates an attempt to execute beyond the gas limit. Within the context of the pvm, ϱ ∈ N G is typically use...

### 4.8. Epochs and Slots

Unlike the YP Ethereum with its proof-of-work consensus system, Jam defines a proof-of-authority consensus mechanism, with the authorized validators presumed to be identified by a set of public keys and decided by a staking mechanism residing within some system hosted by Jam. The staking system is out of scope for the present work; instead there is an api which may be utilized to update these keys, and we presume that whatever logic is needed for the staking system will be introduced and utilize this api as needed. The Safrole mechanism subdivides time following genesis into fixed length epochs with each epoch divided into E = 600 time slots each of uniform length P = 6 seconds, given an epoch period of E ⋅ P = 3600 seconds or one hour. This six-second slot period represents the minimum time between Jam blocks, and through Safrole we aim to strictly minimize forks arising both due to contention within a slot (where two valid blocks may be produced within the same six-second period) and due to contention over multiple slots (where two valid blocks are produced in different time slots but with the same parent). Formally when identifying a timeslot index, we use a natural less than 2 32 (in compute parlance, a 32-bit unsigned integer) indicating the number of six-second timeslots from the Jam Common Era. For use in this context we introduce the set N T : N T ≡ N 2 32 (4.28) This implies that the lifespan of the proposed protocol takes us to mid-August of the year 2840, which with the current course that humanity is on should be ample.

### 4.9. The Core Model and Services

Whereas in the Ethereum Yellow Paper when defining the state machine which is held in consensus amongst all network participants, we presume that all machines maintaining the full network state and contributing to its enlargement—or, at least, hoping to—evaluate all computation. This "everybody does everything" approach might be called the on-chain consensus model. It is unfortunately not scalable, since the network can only process as much logic in consensus that it could hope any individual node is capable of doing itself within any given period of time.

#### 4.9.1. In-core Consensus

In the present work, we achieve scalability of the work done through introducing a second model for such computation which we call the in-core consensus model. In this model, and under normal circumstances, only a subset of the network is responsible for actually executing any given computation and assuring the availability of any input data it relies upon to others. By doing this and assuming a certain amount of computational parallelism within the validator nodes of the network, we are able to scale the amount of computation done in consensus commensurate with the size of the network, and not with the computational power of any single machine. In the present work we expect the network to be able to do upwards of 300 times the amount of computation in-core as that which could be performed by a single machine running the virtual machine at full speed. Since in-core consensus is not evaluated or verified by all nodes on the network, we must find other ways to become adequately confident that the results of the computation are correct, and any data used in determining this is available for a practical period of time. We do this through a crypto-economic game of three stages called guaranteeing, assuring, auditing and, potentially, judging. Respectively, these attach a substantial economic cost to the invalidity of some proposed computation; then a sufficie...

---

## 5. The Header

We must first define the header in terms of its components. The header comprises a parent hash and prior state root (H p and H r), an extrinsic hash H x, a time-slot index H t, the epoch, winning-tickets and offenders markers H e, H w and H o, a Bandersnatch block author index H i and two Bandersnatch signatures; the entropy-yielding vrf signature H v and a block seal H s. Headers may be serialized to an octet sequence with and without the latter seal component using E and E U respectively. Formally: (5.1) H ≡ (H p, H r, H x, H t, H e, H w, H o, H i, H v, H s) The blockchain is a sequence of blocks, each cryptographically referencing some prior block by including a hash derived from the parent's header, all the way back to some first block which references the genesis header. We already presume consensus over this genesis header H 0 and the state it represents defined as σ 0. Excepting the Genesis header, all block headers H have an associated parent header, whose hash is H p. We denote the parent header H − = P (H) : (5.2) H p ∈ H, H p ≡ H (E (P (H))) P is thus defined as being the mapping from one block header to its parent block header. With P, we are able to define the set of ancestor headers A : h ∈ A ⇔ h = H ∨ (∃ i ∈ A ∶ h = P (i)) (5.3) We only require implementations to store headers of ancestors which were authored in the previous L = 24 hours of any block B they wish to validate. The extrinsic hash is a Merkle commitment to the block's extrinsic data, taking care to allow for the possibility of reports to individually have their inclusion proven. Given any block B = (H, E), then formally: H x ∈ H, H x ≡ H (E (H # (a))) (5.4) where a = [ E T (E T), E P (E P), g, E A (E A), E D (E D)] (5.5) and g = E (↕ [(H (w), E 4 (t), ↕ a) S (w, t, a) ← E G ]) (5.6) A block may only be regarded as valid once the timeslot index H t is in the past. It is always strictly greater than that of its parent. Formally: (5.7) H t ∈ N T, P (H) t < H t ∧ H t ⋅ P ≤ ...

### 5.1. The Markers

If not ∅, then the epoch marker specifies key and entropy relevant to the following epoch in case the ticket contest does not complete adequately (a very much unexpected eventuality). Similarly, the winning-tickets marker, if not ∅, provides the series of 600 slot sealing "tickets" for the next epoch (see the next section). Finally, the offenders marker is the sequence of Ed25519 keys of newly misbehaving validators, to be fully explained in section 10. Formally: (5.10) H e ∈ { H, H, ⟦ { H B, H E } ⟧ V } ?, H w ∈ ⟦ C ⟧ E ?, H o ∈ ⟦ H E ⟧ The terms are fully defined in sections 6.6 and 10.

---

## 6. Block Production and Chain Growth

As mentioned earlier, Jam is architected around a hybrid consensus mechanism, similar in nature to that of Polkadot's Babe / Grandpa hybrid. Jam's block production mechanism, termed Safrole after the novel Sassafras production mechanism of which it is a simplified variant, is a stateful system rather more complex than the Nakamoto consensus described in the YP. The chief purpose of a block production consensus mechanism is to limit the rate at which new blocks may be authored and, ideally, preclude the possibility of "forks": multiple blocks with equal numbers of ancestors. To achieve this, Safrole limits the possible author of any block within any given six-second timeslot to a single key-holder from within a pre-specified set of validators. Furthermore, under normal operation, the identity of the key-holder of any future timeslot will have a very high degree of anonymity. As a side effect of its operation, we can generate a high-quality pool of entropy which may be used by other parts of the protocol and is accessible to services running on it. Because of its tightly scoped role, the core of Safrole's state, γ, is independent of the rest of the protocol. It interacts with other portions of the protocol through ι and κ, the prospective and active sets of validator keys respectively; τ, the most recent block's timeslot; and η, the entropy accumulator. The Safrole protocol generates, once per epoch, a sequence of E sealing keys, one for each potential block within a whole epoch. Each block header includes its timeslot index H t (the number of six-second periods since the Jam Common Era began) and a valid seal signature H s, signed by the sealing key corresponding to the timeslot within the aforementioned sequence. Each sealing key is in fact a pseudonym for some validator which was agreed the privilege of authoring a block in the corresponding timeslot. In order to generate this sequence of sealing keys in regular operation, an...

### 6.1. Timekeeping

Here, τ defines the most recent block's slot index, which we transition to the slot index as defined in the block's header: (6.1) τ ∈ N T, τ ′ ≡ H t We track the slot index in state as τ in order that we are able to easily both identify a new epoch and determine the slot at which the prior block was authored. We denote e as the prior's epoch index and m as the prior's slot phase index within that epoch and e ′ and m ′ are the corresponding values for the present block: let e R m = τ E, e ′ R m ′ = τ ′ E (6.2)

### 6.2. Safrole Basic State

We restate γ into a number of components: γ ≡ { γ k, γ z, γ s, γ a } (6.3) γ z is the epoch's root, a Bandersnatch ring root composed with the one Bandersnatch key of each of the next epoch's validators, defined in γ k (itself defined in the next section). γ z ∈ Y R (6.4) Finally, γ a is the ticket accumulator, a series of highest-scoring ticket identifiers to be used for the next epoch. γ s is the current epoch's slot-sealer series, which is either a full complement of E tickets or, in the case of a fallback mode, a series of E Bandersnatch keys: γ a ∈ ⟦ C ⟧ ∶ E, γ s ∈ ⟦ C ⟧ E ∪ ⟦ H B ⟧ E (6.5) Here, C is used to denote the set of tickets, a combination of a verifiably random ticket identifier y and the ticket's entry-index r : C ≡ { y ∈ H, r ∈ N N } (6.6) As we state in section 6.4, Safrole requires that every block header H contain a valid seal H s, which is a Bandersnatch signature for a public key at the appropriate index m of the current epoch's seal-key series, present in state as γ s.

### 6.3. Key Rotation

In addition to the active set of validator keys κ and staging set ι, internal to the Safrole state we retain a pending set γ k. The active set is the set of keys identifying the nodes which are currently privileged to author blocks and carry out the validation processes, whereas the pending set γ k, which is reset to ι at the beginning of each epoch, is the set of keys which will be active in the next epoch and which determine the Bandersnatch ring root which authorizes tickets into the sealing-key contest for the next epoch. ι ∈ ⟦ K ⟧ V, γ k ∈ ⟦ K ⟧ V, κ ∈ ⟦ K ⟧ V, λ ∈ ⟦ K ⟧ V (6.7) We must introduce K, the set of validator key tuples. This is a combination of a set of cryptographic public keys and metadata which is an opaque octet sequence, but utilized to specify practical identifiers for the validator, not least a hardware address. The set of validator keys itself is equivalent to the set of 336-octet sequences. However, for clarity, we divide the sequence into four easily denoted components. For any validator key k, the Bandersnatch key is denoted k b, and is equivalent to the first 32-octets; the Ed25519 key, k e, is the second 32 octets; the bls key denoted k BLS is equivalent to the following 144 octets, and finally the metadata k m is the last 128 octets. Formally: K ≡ Y 336 (6.8) ∀ k ∈ K ∶ k b ∈ H B ≡ k 0 ⋅⋅⋅+ 32 (6.9) ∀ k ∈ K ∶ k e ∈ H E ≡ k 32 ⋅⋅⋅+ 32 (6.10) ∀ k ∈ K ∶ k BLS ∈ Y BLS ≡ k 64 ⋅⋅⋅+ 144 (6.11) ∀ k ∈ K ∶ k m ∈ Y 128 ≡ k 208 ⋅⋅⋅+ 128 (6.12) With a new epoch under regular conditions, validator keys get rotated and the epoch's Bandersnatch key root is updated into γ ′ z : (γ ′ k, κ ′, λ ′, γ ′ z) ≡ { (Φ (ι), γ k, κ, z) if e ′ > e, (γ k, κ, λ, γ z) otherwise } (6.13) where z = O ([ k b S k ← γ ′ k ]) Φ (k) ≡ [ [ 0, 0 ,. .. ] if k e ∈ ψ ′ o, k otherwise ] ∀ k ← k (6.14) Note that on epoch changes the posterior queued validator key set γ ′ k is defined such tha...

### 6.4. Sealing and Entropy Accumulation

The header must contain a valid seal and valid vrf output. These are two signatures both using the current slot's seal key; the message data of the former is the header's serialization omitting the seal component H s, whereas the latter is used as a bias-resistant entropy source and thus its message must already have been fixed: we use the entropy stemming from the vrf of the seal signature. Formally: let i = γ ′ s [ H t ] ↺ ∶ γ ′ s ∈ ⟦ C ⟧ ⇒ { i y = Y (H s), H s ∈ F E U (H) H a ⟨ X T ⌢ η ′ 3 i r ⟩, T = 1 } (6.15) γ ′ s ∈ ⟦ H B ⟧ ⇒ { i = H a, H s ∈ F E U (H) H a ⟨ X F ⌢ η ′ 3 ⟩, T = 0 } (6.16) H v ∈ F [] H a ⟨ X E ⌢ Y (H s)⟩ (6.17) X E = $jam_entropy (6.18) X F = $jam_fallback_seal (6.19) X T = $jam_ticket_seal (6.20) Sealing using the ticket is of greater security, and we utilize this knowledge when determining a candidate block on which to extend the chain, detailed in section 19. We thus note that the block was sealed under the regular security with the boolean marker T. We define this only for the purpose of ease of later specification. In addition to the entropy accumulator η 0, we retain three additional historical values of the accumulator at the point of each of the three most recently ended epochs, η 1, η 2 and η 3. The second-oldest of these η 2 is utilized to help ensure future entropy is unbiased (see equation 6.29) and seed the fallback seal-key generation function with randomness (see equation 6.24). The oldest is used to regenerate this randomness when verifying the seal above (see equations 6.16 and 6.15). η ∈ ⟦ H ⟧ 4 (6.21) η 0 defines the state of the randomness accumulator to which the provably random output of the vrf, the signature over some unbiasable input, is combined each block. η 1, η 2 and η 3 meanwhile retain the state of this accumulator at the end of the three most recently ended epochs in order. η ′ 0 ≡ H (η 0 ⌢ Y (H v)) (6.22) On an epoch transition (id...

### 6.5. The Slot Key Sequence

The posterior slot key sequence γ ′ s is one of three expressions depending on the circumstance of the block. If the block is not the first in an epoch, then it remains unchanged from the prior γ s. If the block signals the next epoch (by epoch index) and the previous block's slot was within the closing period of the previous epoch, then it takes the value of the prior ticket accumulator γ a. Otherwise, it takes the value of the fallback key sequence. Formally: γ ′ s ≡ { Z (γ a) if e ′ = e + 1 ∧ m ≥ Y ∧ S γ a S = E, γ s if e ′ = e, F (η ′ 2, κ ′) otherwise } (6.24) Here, we use Z as the outside-in sequencer function, defined as follows: (6.25) Z ∶ ⟦ C ⟧ E → ⟦ C ⟧ E, s ↦ [ s 0, s S s S − 1, s 1, s S s S − 2 ,. .. ] Finally, F is the fallback key sequence function which selects an epoch's worth of validator Bandersnatch keys (⟦ H B ⟧ E) from the validator key set k using the entropy collected on-chain r : (6.26) F ∶ { { H, ⟦ K ⟧ } → ⟦ H B ⟧ E, { r, k } ↦ k [ E − 1 (H 4 (r ⌢ E 4 (i)))] ↺ b ∀ i ∈ N E }

### 6.6. The Markers

The epoch and winning-tickets markers are information placed in the header in order to minimize data transfer necessary to determine the validator keys associated with any given epoch. They are particularly useful to nodes which do not synchronize the entire state for any given block since they facilitate the secure tracking of changes to the validator key sets using only the chain of headers. As mentioned earlier, the header's epoch marker H e is either empty or, if the block is the first in a new epoch, then a tuple of the next and current epoch randomness, along with a sequence of tuples containing both Bandersnatch keys and Ed25519 keys for each validator defining the validator keys beginning in the next epoch. Formally: H e ≡ { (η 0, η 1, [ { k b, k e } S k ← γ ′ k ]) if e ′ > e, ∅ otherwise } (6.27) The winning-tickets marker H w is either empty or, if the block is the first after the end of the submission period for tickets and if the ticket accumulator is saturated, then the final sequence of ticket identifiers. Formally: H w ≡ { Z (γ a) if e ′ = e ∧ m < Y ≤ m ′ ∧ S γ a S = E, ∅ otherwise } (6.28)

### 6.7. The Extrinsic and Tickets

The extrinsic E T is a sequence of proofs of valid tickets; a ticket implies an entry in our epochal "contest" to determine which validators are privileged to author a block for each timeslot in the following epoch. Tickets specify an entry index together with a proof of ticket's validity. The proof implies a ticket identifier, a high-entropy unbiasable 32-octet sequence, which is used both as a score in the aforementioned contest and as input to the on-chain vrf. Towards the end of the epoch (i.e. Y slots from the start) this contest is closed implying successive blocks within the same epoch must have an empty tickets extrinsic. At this point, the following epoch's seal key sequence becomes fixed. We define the extrinsic as a sequence of proofs of valid tickets, each of which is a tuple of an entry index (a natural number less than N) and a proof of ticket validity. Formally: E T ∈ D { r ∈ N N, p ∈ F̄ [] γ ′ z ⟨ X T ⌢ η ′ 2 r ⟩ } I (6.29) S E T S ≤ { K if m ′ < Y, 0 otherwise } (6.30) We define n as the set of new tickets, with the ticket identifier, a hash, defined as the output component of the Bandersnatch Ring vrf proof: n ≡ [ { y ▸ ▸ Y (i p), r ▸ ▸ i r } S i ← E T ] (6.31) The tickets submitted via the extrinsic must already have been placed in order of their implied identifier. Duplicate identifiers are never allowed lest a validator submit the same ticket multiple times: n = [ x y ∀ x ∈ n ] (6.32) { x y S x ∈ n } ⫰ { x y S x ∈ γ a } (6.33) The new ticket accumulator γ ′ a is constructed by merging new tickets into the previous accumulator value (or the empty sequence if it is a new epoch): (6.34) γ ′ a ≡ → [ x y ∀ x ∈ n ∪ { ∅ if e ′ > e, γ a otherwise } ] E The maximum size of the ticket accumulator is E. On each block, the accumulator becomes the lowest items of the sorted union of tickets from prior accumulator γ a ...

---

## 7. Recent History

We retain in state information on the most recent H blocks. This is used to preclude the possibility of duplicate or out of date work-reports from being submitted. (7.1) β ∈ ⟦ { h ∈ H, b ∈ ⟦ H ? ⟧, s ∈ H, p ∈ D ⟨ H → H ⟩ } ⟧ ∶ H For each recent block, we retain its header hash, its state root, its accumulation-result mmr and the corresponding work-package hashes of each item reported (which is no more than the total number of cores, C = 341). During the accumulation stage, a value with the partial transition of this state is provided which contains the update for the newly-known roots of the parent block: (7.2) β † ≡ β except β † [S β S − 1 ] s = H r We define an item n comprising the new block's header hash, its accumulation-result Merkle tree root and the set of work-reports made into it (for which we use the guarantees extrinsic, E G). Note that the accumulation-result tree root r is derived from C (defined in section 12) using the basic binary Merklization function M B (defined in appendix E) and appending it using the mmr append function A (defined in appendix E.2) to form a Merkle mountain range. (7.3) let r = M B ([ s ∧ E 4 (s) ⌢ E (h) S (s, h) ∈ C ], H K) let b = A (last ([[]] ⌢ [ x b S x ← β ]), r, H K) let p = {((g w) s) h ↦ ((g w) s) e S g ∈ E G } let n = { p, h ▸ ▸ H (H), b, s ▸ ▸ H 0 } The state-trie root is as being the zero hash, H 0 which while inaccurate at the end state of the block β ′, it is nevertheless safe since β ′ is not utilized except to define the next block's β †, which contains a corrected value for this. The final state transition is then: (7.4) β ′ ≡ ← β † n H

---

## 8. Authorization

We have previously discussed the model of work-packages and services in section 4.9, however we have yet to make a substantial discussion of exactly how some coretime resource may be apportioned to some work-package and its associated service. In the YP Ethereum model, the underlying resource, gas, is procured at the point of introduction on-chain and the purchaser is always the same agent who authors the data which describes the work to be done (i.e. the transaction). Conversely, in Polkadot the underlying resource, a parachain slot, is procured with a substantial deposit for typically 24 months at a time and the procurer, generally a parachain team, will often have no direct relation to the author of the work to be done (i.e. a parachain block). On a principle of flexibility, we would wish Jam capable of supporting a range of interaction patterns both Ethereum-style and Polkadot-style. In an effort to do so, we introduce the authorization system, a means of disentangling the intention of usage for some coretime from the specification and submission of a particular workload to be executed on it. We are thus able to disassociate the purchase and assignment of coretime from the specific determination of work to be done with it, and so are able to support both Ethereum-style and Polkadot-style interaction patterns.

### 8.1. Authorizers and Authorizations

The authorization system involves three key concepts: Authorizers, Tokens and Traces. A Token is simply a piece of opaque data to be included with a work-package to help make an argument that the work-package should be authorized. Similarly, a Trace is a piece of opaque data which helps characterize or describe some successful authorization. An Authorizer meanwhile, is a piece of logic which executes within some pre-specified and well-known computational limits and determines whether a work-package—including its Token—is authorized for execution on some particular core and yields a Trace on success. Authorizers are identified as the hash of their pvm code concatenated with their Configuration blob, the latter being, like Tokens and Traces, opaque data meaningful to the pvm code. The process by which work-packages are determined to be authorized (or not) is not the competence of on-chain logic and happens entirely in-core and as such is discussed in section 14.3. However, on-chain logic must identify each set of authorizers assigned to each core in order to verify that a work-package is legitimately able to utilize that resource. It is this subsystem we will now define.

### 8.2. Pool and Queue

We define the set of authorizers allowable for a particular core c as the authorizer pool α [ c ]. To maintain this value, a further portion of state is tracked for each core: the core's current authorizer queue φ [ c ], from which we draw values to fill the pool. Formally: (8.1) α ∈ C⟦ H ⟧ ∶ O H C, φ ∈ C⟦ H ⟧ Q H C Note: The portion of state φ may be altered only through an exogenous call made from the accumulate logic of an appropriately privileged service. The state transition of a block involves placing a new authorization into the pool from the queue: ∀ c ∈ N C ∶ α ′ [ c ] ≡ ← F (c) φ ′ [ c ][ H t ] ↺ O (8.2) F (c) ≡ { α [ c ] m {(g w) a } if ∃ g ∈ E G ∶ (g w) c = c, α [ c ] otherwise } (8.3) Since α ′ is dependent on φ ′, practically speaking, this step must be computed after accumulation, the stage in which φ ′ is defined. Note that we utilize the guarantees extrinsic E G to remove the oldest authorizer which has been used to justify a guaranteed work-package in the current block. This is further defined in equation 11.23.

---

## 9. Service Accounts

As we already noted, a service in Jam is somewhat analogous to a smart contract in Ethereum in that it includes amongst other items, a code component, a storage component and a balance. Unlike Ethereum, the code is split over two isolated entry-points each with their own environmental conditions; one, refinement, is essentially stateless and happens in-core, and the other, accumulation, which is stateful and happens on-chain. It is the latter which we will concern ourselves with now. Service accounts are held in state under δ, a partial mapping from a service identifier N S into a tuple of named elements which specify the attributes of the service relevant to the Jam protocol. Formally: N S ≡ N 2 32 (9.1) δ ∈ D ⟨ N S → A ⟩ (9.2) The service account is defined as the tuple of storage dictionary s, preimage lookup dictionaries p and l, code hash c, and balance b as well as the two code gas limits g & m. Formally: A ≡ { s ∈ D ⟨ H → Y ⟩, p ∈ D ⟨ H → Y ⟩, l ∈ D ⟨ { H, N L } → ⟦ N T ⟧ ∶ 3 ⟩, c ∈ H, b ∈ N B, g ∈ N G, m ∈ N G } (9.3) Thus, the balance of the service of index s would be denoted δ [ s ] b and the storage item of key k ∈ H for that service is written δ [ s ] s [ k ].

### 9.1. Code and Gas

The code and associated metadata of a service account is identified by a hash which, if the service is to be functional, must be present within its preimage lookup (see section 9.2). We thus define the actual code c and metadata m : ∀ a ∈ A ∶ E (↕ a m, a c) ≡ { a p [ a c ] if a c ∈ a p, ∅ otherwise } (9.4) There are three entry-points in the code: 0 refine : Refinement, executed in-core and stateless. 1 accumulate : Accumulation, executed on-chain and stateful. 2 on_transfer : Transfer handler, executed on-chain and stateful. Whereas the first, executing in-core, is described in more detail in section 14.3, the latter two are defined in the present section. As stated in appendix A, execution time in the Jam virtual machine is measured deterministically in units of gas, represented as a natural number less than 2 64 and formally denoted N G. We may also use Z G to denote the set Z − 2 63 ... 2 63 if the quantity may be negative. There are two limits specified in the account, g, the minimum gas required in order to execute the Accumulate entry-point of the service's code, and m, the minimum required for the On Transfer entry-point.

### 9.2. Preimage Lookups

In addition to storing data in arbitrary key/value pairs available only on-chain, an account may also solicit data to be made available also in-core, and thus available to the Refine logic of the service's code. State concerning this facility is held under the service's p and l components. There are several differences between preimage-lookups and storage. Firstly, preimage-lookups act as a mapping from a hash to its preimage, whereas general storage maps arbitrary keys to values. Secondly, preimage data is supplied extrinsically, whereas storage data originates as part of the service's accumulation. Thirdly preimage data, once supplied, may not be removed freely; instead it goes through a process of being marked as unavailable, and only after a period of time may it be removed from state. This ensures that historical information on its existence is retained. The final point especially is important since preimage data is designed to be queried in-core, under the Refine logic of the service's code, and thus it is important that the historical availability of the preimage is known. We begin by reformulating the portion of state concerning our data-lookup system. The purpose of this system is to provide a means of storing static data on-chain such that it may later be made available within the execution of any service code as a function accepting only the hash of the data and its length in octets. During the on-chain execution of the Accumulate function, this is trivial to achieve since there is inherently a state which all validators verifying the block necessarily have complete knowledge of, i.e. σ. However, for the in-core execution of Refine, there is no such state inherently available to all validators; we t...

### 9.3. Account Footprint and Threshold Balance

We define the dependent values i and o as the storage footprint of the service, specifically the number of items in storage and the total number of octets used in storage. They are defined purely in terms of the storage map of a service, and it must be assumed that whenever a service's storage is changed, these change also. Furthermore, as we will see in the account serialization function in section C, these are expected to be found explicitly within the Merklized state data. Because of this we make explicit their set. We may then define a second dependent term t, the minimum, or threshold, balance needed for any given service account in terms of its storage footprint. ∀ a ∈ V (δ) ∶ { a i ∈ N 2 32 ≡ 2 ⋅ S a l S + S a s S, a o ∈ N 2 64 ≡ ∑ (h,z) ∈ K (a l) 81 + z + ∑ x ∈ V (a s) 32 + S x S, a t ∈ N B ≡ B S + B I ⋅ a i + B L ⋅ a o } (9.8)

### 9.4. Service Privileges

Up to three services may be recognized as privileged. The portion of state in which this is held is denoted χ and has three service index components together with a gas limit. The first, χ m, is the index of the manager service which is the service able to effect an alteration of χ from block to block. The following two, χ a and χ v, are each the indices of services able to alter φ and ι from block to block. Finally, χ g is a small dictionary containing the indices of services which automatically accumulate in each block together with a basic amount of gas with which each accumulates. Formally: χ ≡ { χ m ∈ N S, χ a ∈ N S, χ v ∈ N S, χ g ∈ D ⟨ N S → N G ⟩ } (9.9)

---

## 10. Disputes, Verdicts and Judgments

Jam provides a means of recording judgments: consequential votes amongst most of the validators over the validity of a work-report (a unit of work done within Jam, see section 11). Such collections of judgments are known as verdicts. Jam also provides a means of registering offenses, judgments and guarantees which dissent with an established verdict. Together these form the disputes system. The registration of a verdict is not expected to happen very often in practice, however it is an important security backstop for removing and banning invalid work-reports from the processing pipeline as well as removing troublesome keys from the validator set where there is consensus over their malfunction. It also helps coordinate nodes to revert chain-extensions containing invalid work-reports and provides a convenient means of aggregating all offending validators for punishment in a higher-level system. Judgement statements come about naturally as part of the auditing process and are expected to be positive, further affirming the guarantors' assertion that the work-report is valid. In the event of a negative judgment, then all validators audit said work-report and we assume a verdict will be reached. Auditing and guaranteeing are off-chain processes properly described in sections 14 and 17. A judgment against a report implies that the chain is already reverted to some point prior to the accumulation of said report, usually forking at the block immediately prior to that at which accumulation happened. The specific strategy for chain selection is described fully in section 19. Authoring a block with a non-positive verdict has the effect of cancelling its imminent accumulation, as can be seen in equation 10.15. Registering a verdict also has the effect of placing a permanent record of the event on-chain and allowing any offending keys to be placed on-chain both immediately or in forth...

### 10.1. The State

The disputes state includes four items, three of which concern verdicts: a good-set (ψ g), a bad-set (ψ b) and a wonky-set (ψ w) containing the hashes of all work-reports which were respectively judged to be correct, incorrect or that it appears impossible to judge. The fourth item, the punish-set (ψ o), is a set of Ed25519 keys representing validators which were found to have misjudged a work-report. (10.1) ψ ≡ { ψ g, ψ b, ψ w, ψ o }

### 10.2. Extrinsic

The disputes extrinsic, E D, may contain one or more verdicts v as a compilation of judgments coming from exactly two-thirds plus one of either the active validator set or the previous epoch's validator set, i.e. the Ed25519 keys of κ or λ. Additionally, it may contain proofs of the misbehavior of one or more validators, either by guaranteeing a work-report found to be invalid (culprits, c), or by signing a judgment found to be contradiction to a work-report's validity (faults, f). Both are considered a kind of offense. Formally: (10.2) E D ≡ (v, c, f) where v ∈ E { H, τ E − N 2, ⟦ { { ⊺, ⊥ }, N V, E } ⟧ ⌊ 2 ~ 3 V ⌋ + 1 } J and c ∈ ⟦ H, H E, E ⟧, f ∈ ⟦ H, { ⊺, ⊥ }, H E, E ⟧ The signatures of all judgments must be valid in terms of one of the two allowed validator key-sets, identified by the verdict's second term which must be either the epoch index of the prior state or one less. Formally: ∀ (r, a, j) ∈ v, ∀ (v, i, s) ∈ j ∶ s ∈ E k [ i ] e ⟨ X v ⌢ r ⟩ where k = { κ if a = τ E, λ otherwise } (10.3) X ⊺ ≡ $jam_valid, X ⊥ ≡ $jam_invalid (10.4) Offender signatures must be similarly valid and reference work-reports with judgments and may not report keys which are already in the punish-set...

### 10.3. Header

The offenders markers must contain exactly the keys of all new offenders, respectively. Formally: H o ≡ [ k S (r, k, s) ∈ c ] ⌢ [ k S (r, v, k, s) ∈ f ] (10.20)

---

## 11. Reporting and Assurance

Reporting and assurance are the two on-chain processes we do to allow the results of in-core computation to make their way into the service state singleton, δ. A work-package, which comprises several work items, is transformed by validators acting as guarantors into its corresponding work-report, which similarly comprises several work-digests and then presented on-chain within the guarantees extrinsic. At this point, the work-package is erasure coded into a multitude of segments and each segment distributed to the associated validator who then attests to its availability through an assurance placed on-chain. After enough assurances the work-report is considered available, and the work-digests transform the state of their associated service by virtue of accumulation, covered in section 12. The report may also be timed-out, implying it may be replaced by another report without accumulation. From the perspective of the work-report, therefore, the guarantee happens first and the assurance afterwards. However, from the perspective of a block's state-transition, the assurances are best processed first since each core may only have a single work-report pending its package becoming available at a time. Thus, we will first cover the transition arising from processing the availability assurances followed by the work-report guarantees. This synchroneity can be seen formally through the requirement of an intermediate state ρ ‡, utilized later in equation 11.29.

### 11.1. State

The state of the reporting and availability portion of the protocol is largely contained within ρ, which tracks the work-reports which have been reported but are not yet known to be available to a super-majority of validators, together with the time at which each was reported. As mentioned earlier, only one report may be assigned to a core at any given time. Formally: (11.1) ρ ∈ ⟦ { w ∈ W, t ∈ N T } ? ⟧ C As usual, intermediate and posterior values (ρ †, ρ ‡, ρ ′) are held under the same constraints as the prior value.

#### 11.1.1. Work Report

A work-report, of the set W, is defined as a tuple of the work-package specification, s ; the refinement context, x ; the core-index (i.e. on which the work is done), c ; as well as the authorizer hash a and trace o ; a segment-root lookup dictionary l ; the gas consumed during the Is-Authorized invocation, g ; and finally the work-digests r which comprise the results of the evaluation of each of the items in the package together with some associated data. Formally: (11.2) W ≡ { s ∈ S, x ∈ X, c ∈ N C, a ∈ H, o ∈ Y, l ∈ D ⟨ H → H ⟩, r ∈ ⟦ L ⟧ 1 ∶ I, g ∈ N G, } We limit the sum of the number of items in the segment-root lookup dictionary and the number of prerequisites to J = 8 : (11.3) ∀ w ∈ W ∶ S w l S + S(w x) p S ≤ J

#### 11.1.2. Refinement Context

A refinement context, denoted by the set X, describes the context of the chain at the point that the report's corresponding work-package was evaluated. It identifies two historical blocks, the anchor, header hash a along with its associated posterior state-root s and posterior Beefy root b ; and the lookup-anchor, header hash l and of timeslot t. Finally, it identifies the hash of any prerequisite work-packages p. Formally: (11.4) X ≡ { a ∈ H, s ∈ H, b ∈ H, l ∈ H, t ∈ N T, p ∈ { H } }

#### 11.1.3. Availability

We define the set of availability specifications, S, as the tuple of the work-package's hash h, an auditable wo...

### 11.2. Package Availability Assurances

We first define ρ ‡, the intermediate state to be utilized next in section 11.4 as well as W, the set of available work-reports, which will we utilize later in section 12. Both require the integration of information from the assurances extrinsic E A.

#### 11.2.1. The Assurances Extrinsic

The assurances extrinsic is a sequence of assurance values, at most one per validator. Each assurance is a sequence of binary values (i.e. a bitstring), one per core, together with a signature and the index of the validator who is assuring. A value of 1 (or ⊺, if interpreted as a Boolean) at any given index implies that the validator assures they are contributing to its availability. Formally: E A ∈ ⟦ { a ∈ H, f ∈ B C, v ∈ N V, s ∈ E } ⟧ ∶ V (11.10) The assurances must all be anchored on the parent and ordered by validator index: ∀ a ∈ E A ∶ a a = H p (11.11) ∀ i ∈ { 1. .. S E A S} ∶ E A [ i − 1 ] v < E A [ i ] v (11.12) The signature must be one whose public key is that of the validator assuring and whose message is the serialization of the parent hash H p and the aforementioned bitstring: ∀ a ∈ E A ∶ a s ∈ E κ [ a v ] e ⟨ X A ⌢ H (E (H p, a f))⟩ (11.13) X A ≡ $jam_available (11.14) A bit may only be set if the corresponding core has a report pending availability on it: (11.15) ∀ a ∈ E A, c ∈ N C ∶ a f [ c ] ⇒ ρ † [ c ] ≠ ∅

#### 11.2.2. Available Reports

A work-report is said to become available if and only if there are a clear 2 / 3 supermajority of validators who have marked its core as set within the block's assurance extrinsic. Formally, we define the sequence of newly available work-reports W as: W ≡ [ ρ † [ c ] w ∣ c ← N C, ∑ a ∈ E A a f [ c ] > 2 ~ 3 V ] (11.16) This value is utilized in the definition of both δ ′ and ρ ‡ which we will define presently as equivalent to ρ † except for the removal of items which are either now available or have timed out: ∀ c ∈ N C ∶ ρ ‡ [ c ] ≡ { ∅ i...

### 11.3. Guarantor Assignments

Every block, each core has three validators uniquely assigned to guarantee work-reports for it. This is borne out with V = 1,023 validators and C = 341 cores, since V ~ C = 3. The core index assigned to each of the validators, as well as the validators' Ed25519 keys are denoted by G : (11.18) G ∈ (⟦ N C ⟧ N V, ⟦ H K ⟧ N V) We determine the core to which any given validator is assigned through a shuffle using epochal entropy and a periodic rotation to help guard the security and liveness of the network. We use η 2 for the epochal entropy rather than η 1 to avoid the possibility of fork-magnification where uncertainty about chain state at the end of an epoch could give rise to two established forks before it naturally resolves. We define the permute function P, the rotation function R and finally the guarantor assignments G as follows: R (c, n) ≡ [(x + n) mod C S x ← c ] (11.19) P (e, t) ≡ R ( F ( C ⋅ i V ∀ i ← N V , e , t mod E R ) (11.20) G ≡ (P (η ′ 2, τ ′), Φ (κ ′)) (11.21) We also define G ∗, which is equivalent to the value G as it would have been under the previous rotation: (11.22) let (e, k) = { (η ′ 2, κ ′) if τ ′ − R E = τ ′ E, (η ′ 3, λ ′) otherwise } G ∗ ≡ (P (e, τ ′ − R), Φ (k))

### 11.4. Work Report Guarantees

We begin by defining the guarantees extrinsic, E G, a series of guarantees, at most one for each core, each of which is a tuple of a work-report, a credential a and its corresponding timeslot t. The core index of each guarantee must be unique and guarantees must be in ascending order of this. Formally: E G ∈ C { w ∈ W, t ∈ N T, a ∈ ⟦ { N V, E } ⟧ 2 ∶ 3 } H ∶ C (11.23) E G = [(g w) c ∀ g ∈ E G ] (11.24) The credential is a sequence of two or three tuples of a unique validator index and a signature. Credentials must be ordered by their validator index: ∀ g ∈ E G ∶ g a = [ v ∀ { v, s } ∈ g a ] (11.25) The signature must be one whose public key is that of the validator identified in the credential, and whose message is the serialization of the hash of the work-report. The signing validators must be assigned to the core in question in either this block G if the timeslot for the guarantee is in the same rotation as this block's timeslot, or in the most recent previous set of assignments, G ∗ : ∀ (w, t, a) ∈ E G, ∀ (v, s) ∈ a ∶ { s ∈ E (k v) e ⟨ X G ⌢ H (w)⟩, c v = w c ∧ R (τ ′ ~ R − 1) ≤ t ≤ τ ′, k ∈ R ⇔ ∃ (w, t, a) ∈ E G, ∃ (v, s) ∈ a ∶ k = (k v) e where (c, k) = { G if τ ′ R = t R, G ∗ otherwise } } (11.26) X G ≡ $jam_guarantee (11.27) We note that the Ed25519 key of each validator whose signature is in a credential is placed in the reporters set R. This is utilized by the validator activity statistics bookkeeping system section 13. We denote w to be the set of work-reports in the present extrinsic E : let w = { g w S g ∈ E G } (11.28) No reports may be placed on cores with a report pending availability on it. A report is valid only if the authorizer hash is present in the authorizer pool of the core on which the work is reported. Formally: (11.29) ∀ w ∈ w ∶ ρ ‡ [ w c ] = ∅ ∧ w a ∈ α [ w c ] We require that the gas allotted for accumulation of each work-digest in each work-report respects i...

### 11.5. Transitioning for Reports

We define ρ ′ as being equivalent to ρ ‡, except where the extrinsic replaced an entry. In the case an entry is replaced, the new value includes the present time τ ′ allowing for the value to be replaced without respect to its availability once sufficient time has elapsed (see equation 11.29). (11.43) ∀ c ∈ N C ∶ ρ ′ [ c ] ≡ { { w, t ▸ ▸ τ ′ } if ∃ { c, w, a } ∈ E G, ρ ‡ [ c ] otherwise } This concludes the section on reporting and assurance. We now have a complete definition of ρ ′ together with W to be utilized in section 12, describing the portion of the state transition happening once a work-report is guaranteed and made available.

---

## 12. Accumulation

Accumulation may be defined as some function whose arguments are W and δ together with selected portions of (at times partially transitioned) state and which yields the posterior service state δ ′ together with additional state elements ι ′, φ ′ and χ ′. The proposition of accumulation is in fact quite simple: we merely wish to execute the Accumulate logic of the service code of each of the services which has at least one work-digest, passing to it relevant data from said digests together with useful contextual information. However, there are three main complications. Firstly, we must define the execution environment of this logic and in particular the host functions available to it. Secondly, we must define the amount of gas to be allowed for each service's execution. Finally, we must determine the nature of transfers within Accumulate which, as we will see, leads to the need for a second entry-point, on-transfer.

### 12.1. History and Queuing

Accumulation of a work-package/work-report is deferred in the case that it has a not-yet-fulfilled dependency and is cancelled entirely in the case of an invalid dependency. Dependencies are specified as work-package hashes and in order to know which work-packages have been accumulated already, we maintain a history of what has been accumulated. This history, ξ, is sufficiently large for an epoch worth of work-reports. Formally: ξ ∈ ⟦{ H }⟧ E (12.1) © ξ ≡ ⋃ x ∈ ξ (x) (12.2) We also maintain knowledge of ready (i.e. available and/or audited) but not-yet-accumulated work-reports in the state item ϑ. Each of these were made available at most one epoch ago but have or had unfulfilled dependencies. Alongside the work-report itself, we retain its unaccumulated dependencies, a set of work-package hashes. Formally: ϑ ∈ ⟦⟦(W, { H })⟧⟧ E (12.3) The newly available work-reports, W, are partitioned into two sequences based on the condition of having zero prerequisite work-reports. Those meeting the condition, W !, are accumulated immediately. Those not, W Q, are for queued execution. Formally: W ! ≡ [ w S w ← W, S(w x) p S = 0 ∧ w l = {}] (12.4) W Q ≡ E ([ D (w) S w ← W, S(w x) p S > 0 ∨ w l ≠ {}], © ξ) (12.5) D (w) ≡ (w, {(w x) p } ∪ K (w l)) (12.6) We define the queue-editing function E, which is essentially a mutator function for items such as those of ϑ, parameterized by sets of now-accumulated work-package hashes (those in ξ). It is used to update queues of work-reports when some of them are accumulated. Functionally, it removes all entries whose work-report's hash is in the set provided as a parameter, and removes any dependencies which appear in said set. Formally: (12.7) E ∶ { (⟦(W, { H })⟧, { H }) → ⟦(W, { H })⟧, (r, x) ↦ (w, d ∖ x) ∀ (w, d) ← r, (w s) h ∉ x } We further define the accumulation priority queue function Q, which provides the sequence of work-reports which are accumulatable given a set of not-yet-accu...

### 12.2. Execution

We work with a limited amount of gas per block and therefore may not be able to process all items in W ∗ in a single block. There are two slightly antagonistic factors allowing us to optimize the amount of work-items, and thus work-reports, accumulated in a single block: Firstly, while we have a well-known gas-limit for each work-item to be accumulated, accumulation may still result in a lower amount of gas used. Only after a work-item is accumulated can it be known if it uses less gas than the advertised limit. This implies a sequential execution pattern. Secondly, since pvm setup cannot be expected to be zero-cost, we wish to amortize this cost over as many work-items as possible. This can be done by aggregating work-items associated with the same service into the same pvm invocation. This implies a non-sequential execution pattern. We resolve this by defining a function ∆ + which accumulates work-reports sequentially, and which itself utilizes a function ∆ ∗ which accumulates work-reports in a non-sequential, service-aggregated manner. Only once all such accumulation is executed do we integrate the results and thus define the relevant posterior state items. In doing so we also integrate the consequences of any deferred-transfers implied by accumulation. Our formalisms begin by defining U as a characterization of (i.e. values capable of representing) state components which are both needed and mutable by the accumulation process. This comprises the service accounts state (as in δ), the upcoming validator keys ι, the queue of authorizers φ and the privileges state χ. Formally: (12.13) U ≡ ( d ∈ D ⟨ N S → A ⟩, i ∈ ⟦ K ⟧ V, q ∈ C⟦ H ⟧ Q H C, x ∈ (N S, N S, N S, D ⟨ N S → N G ⟩) ) We denote the set characterizing a deferred transfer as T, noting that a transfer includes a memo component m of W T = 128 octets, together with the service index of the sender s, the service index of the receiver...

### 12.3. Deferred Transfers and State Integration

Given the result of the top-level ∆ +, we may define the posterior state χ ′, φ ′ and ι ′ as well as the first intermediate state of the service-accounts δ † and the Beefy commitment map C : let g = max ( G T, G A ⋅ C + ∑ x ∈ V (χ g) (x) ) (12.21) let (n, o, t, C, u) = ∆ + (g, W ∗, (χ, δ, ι, φ), χ g) (12.22) (χ ′, δ †, ι ′, φ ′) ≡ o (12.23) We compose I, our accumulation statistics, which is a mapping from the service indices which were accumulated to the amount of gas used throughout accumulation and the number of work-items accumulated. Formally: I ∈ D ⟨ N S → { N G, N } ⟩ (12.24) I ≡ s ↦ { ∑ (s,u) ∈ u (u), S N (s)S } ∀ N (s) ≠ [] (12.25) where N (s) ≡ r S w ← W ∗ ...n, r ← w r, r s = s (12.26) Note that the accumulation commitment map C is a set of pairs of indices of the output-yielding accumulated services to their accumulation result. This is utilized in equation 7.3, when determining the accumulation-result tree root for the present block, useful for the Beefy protocol. We have denoted the sequence of implied transfers as t, ordered internally according to the source service's execution. We define a selection function R, which maps a sequence of deferred transfers and a desired destination service index into the sequence of transfers targeting said service, ordered primarily according to the source service index and secondarily their order within t. Formally: (12.27) R ∶ (⟦ T ⟧, N S) → ⟦ T ⟧, (t, d) ↦ [ t S s ← N S, t ← t, t s = s, t d = d ] The second intermediate state δ ‡ may then be defined with all the deferred effects of the transfers applied: x = { s ↦ Ψ T (δ †, τ ′, s, R (t, s)) S (s ↦ a) ∈ δ † } (12.28) δ ‡ ≡ { s ↦ a S (s ↦ { a, u }) ∈ x } (12.29) Furthermore we build the deferred transfers statistics value X as the number of transfers and the total gas used in transfer processing for each destination service inde...

### 12.4. Preimage Integration

After accumulation, we must integrate all preimages provided in the lookup extrinsic to arrive at the posterior account state. The lookup extrinsic is a sequence of pairs of service indices and data. These pairs must be ordered and without duplicates (equation 12.36 requires this). The data must have been solicited by a service but not yet provided in the prior state. Formally: E P ∈ ⟦ { N S, Y } ⟧ (12.35) E P = [ i ∀ i ∈ E P ] (12.36) R (d, s, h, l) ⇔ h ∉ d [ s ] p ∧ d [ s ] l [ { h, l } ] = [] (12.37) ∀ { s, p } ∈ E P ∶ R (δ, s, H (p), S p S) (12.38) We disregard, without prejudice, any preimages which due to the effects of accumulation are no longer useful. We define δ ′ as the state after the integration of the still-relevant preimages: let P = {(s, p) S { s, p } ∈ E P, R (δ ‡, s, H (p), S p S)} (12.39) δ ′ = δ ‡ ex. ∀ { s, p } ∈ P ∶ { δ ′ [ s ] p [ H (p)] = p, δ ′ [ s ] l [ H (p), S p S] = [ τ ′ ] } (12.40)

---

## 13. Statistics

### 13.1. Validator Activity

The Jam chain does not explicitly issue rewards—we leave this as a job to be done by the staking subsystem (in Polkadot's case envisioned as a system parachain—hosted without fees—in the current imagining of a public Jam network). However, much as with validator punishment information, it is important for the Jam chain to facilitate the arrival of information on validator activity in to the staking subsystem so that it may be acted upon. Such performance information cannot directly cover all aspects of validator activity; whereas block production, guarantor reports and availability assurance can easily be tracked on-chain, Grandpa, Beefy and auditing activity cannot. In the latter case, this is instead tracked with validator voting activity: validators vote on their impression of each other's efforts and a median may be accepted as the truth for any given validator. With an assumption of 50% honest validators, this gives an adequate means of oraclizing this information. The validator statistics are made on a per-epoch basis and we retain one record of completed statistics together with one record which serves as an accumulator for the present epoch. Both are tracked in π, which is thus a sequence of two elements, with the first being the accumulator and the second the previous epoch's statistics. For each epoch we track a performance record for each validator: π ≡ { π V, π L, π C, π S } (13.1) (π V, π L) ∈ ⟦ { b ∈ N, t ∈ N, p ∈ N, d ∈ N, g ∈ N, a ∈ N } ⟧ 2 V (13.2) The six validator statistics we track are: b : The number of blocks produced by the validator. t : The number of tickets introduced by the validator. p : The number of preimages introduced by the validator. d : The total number of octets across all preimages introduced by the validator. g : The number of reports guaranteed by the validator. a : The number of availability assurances made by the validator. The objective statistics are updated in line with their descriptio...

### 13.2. Cores and Services

The other two components of statistics are the core and service activity statistics. These are tracked only on a per-block basis unlike the validator statistics which are tracked over the whole epoch. π C ∈ F { d ∈ N, p ∈ N, i ∈ N, e ∈ N, z ∈ N, x ∈ N, b ∈ N, u ∈ N G } K C (13.6) π S ∈ D ⟨ N S → { p ∈ (N, N), r ∈ (N, N G), i ∈ N, e ∈ N, z ∈ N, x ∈ N, a ∈ (N, N G), t ∈ (N, N G) } ⟩ (13.7) The core statistics are updated using several intermediate values from across the overall state-transition function; w, the incoming work-reports, as defined in 11.28 and W, the newly available work-reports, as defined in 11.16. We define the statistics as follows: ∀ c ∈ N C ∶ π ′ C [ c ] ≡ { i ▸ ▸ R (c) i, x ▸ ▸ R (c) x, z ▸ ▸ R (c) z, e ▸ ▸ R (c) e, u ▸ ▸ R (c) u, b ▸ ▸ R (c) b, d ▸ ▸ D (c), p ▸ ▸ ∑ a ∈ E A a f [ c ] } (13.8) where R (c ∈ N C) ≡ ∑ r ∈ w r ,w ∈ w ,w c = c (r i, r x, r z, r e, r u, b ▸ ▸ (w s) l) (13.9) and D (c ∈ N C) ≡ ∑ w ∈ W ,w c = c (w s) l + W G ⌈(w s) n 65 ~ 64 ⌉ (13.10) Finally, the service statistics are updated using the same intermediate values as the core statistics, but with a different set of calculations...

---

## 14. Work Packages and Work Reports

### 14.1. Honest Behavior

We have so far specified how to recognize blocks for a correctly transitioning Jam blockchain. Through defining the state transition function and a state Merklization function, we have also defined how to recognize a valid header. While it is not especially difficult to understand how a new block may be authored for any node which controls a key which would allow the creation of the two signatures in the header, nor indeed to fill in the other header fields, readers will note that the contents of the extrinsic remain unclear. We define not only correct behavior through the creation of correct blocks but also honest behavior, which involves the node taking part in several off-chain activities. This does have analogous aspects within YP Ethereum, though it is not mentioned so explicitly in said document: the creation of blocks along with the gossiping and inclusion of transactions within those blocks would all count as off-chain activities for which honest behavior is helpful. In Jam's case, honest behavior is well-defined and expected of at least 2 ~ 3 of validators. Beyond the production of blocks, incentivized honest behavior includes: the guaranteeing and reporting of work-packages, along with chunking and distribution of both the chunks and the work-package itself, discussed in section 15; assuring the availability of work-packages after being in receipt of their data; determining which work-reports to audit, fetching and auditing them, and creating and distributing judgments appropriately based on the outcome of the audit; submitting the correct amount of auditing work seen being done by other validators, discussed in section 13.

### 14.2. Segments and the Manifest

Our basic erasure-coding segment size is W E = 684 octets, derived from the fact we wish to be able to reconstruct even should almost two-thirds of our 1023 participants be malicious or incapacitated, the 16-bit Galois field on which the erasure-code is based and the desire to efficiently support encoding data of close to, but no less than, 4 kb. Work-packages are generally small to ensure guarantors need not invest a lot of bandwidth in order to discover whether they can get paid for their evaluation into a work-report. Rather than having much data inline, they instead reference data through commitments. The simplest commitments are extrinsic data. Extrinsic data are blobs which are being introduced into the system alongside the work-package itself generally by the work-package builder. They are exposed to the Refine logic as an argument. We commit to them through including each of their hashes in the work-package. Work-packages have two other types of external data associated with them: A cryptographic commitment to each imported segment and finally the number of segments which are exported.

#### 14.2.1. Segments, Imports and Exports

The ability to communicate large amounts of data from one work-package to some subsequent work-package is a key feature of the Jam availability system. An export segment, defined as the set G, is an octet sequence of fixed length W G = 4104. It is the smallest datum which may individually be imported from—or exported to—the long-term D 3 L during the Refine function of a work-package. Being an exact multiple of the erasure-coding piece size ensures that the data segments of work-package can be efficiently placed in the D 3 L system. (14.1) G ≡ Y W G Exported segments are data which are generated through the execution of the Refine logic and thus are a side effect of transforming the work-package into a work-report. Since their data is deterministic ba...

### 14.3. Packages and Items

We begin by defining a work-package, of set P, and its constituent work-items, of set I. A work-package includes a simple blob acting as an authorization token j, the index of the service which hosts the authorization code h, an authorization code hash u and a configuration blob p, a context x and a sequence of work items w : (14.2) P ≡ { j ∈ Y, h ∈ N S, u ∈ H, p ∈ Y, x ∈ X, w ∈ ⟦ I ⟧ 1 ∶ I } A work item includes: s the identifier of the service to which it relates, the code hash of the service at the time of reporting h (whose preimage must be available from the perspective of the lookup anchor block), a payload blob y, gas limits for Refinement and Accumulation g & a, and the three elements of its manifest, a sequence of imported data segments i which identify a prior exported segment through an index and the identity of an exporting work-package, x, a sequence of blob hashes and lengths to be introduced in this block (and which we assume the validator knows) and e the number of data segments exported by this work item. (14.3) I ≡ { s ∈ N S, h ∈ H, y ∈ Y, g ∈ N G, a ∈ N G, e ∈ N, i ∈ C { H ∪ (H ⊞), N } H, x ∈ ⟦(H, N)⟧ } Note that an imported data segment's work-package is identified through the union of sets H and a tagged variant H ⊞. A value drawn from the regular H implies the hash value is of the segment-root containing the export, whereas a value drawn from H ⊞ implies the hash value is the hash of the exporting work-package. In the latter case it must be converted into a segment-root by the guarantor and this conversion reported in the work-report for on-chain validation. We limit the total number of exported items to W X = 3072, the total number of imported items to W M = 3072, and the total number of extrinsics to T = 128 : (14.4) ∀ p ∈ P ∶ ∑ w ∈ p w w e ≤ W X ∧ ∑ w ∈ p w S w i S ≤ W M ∧ ∑ w ∈ p w S w x S ≤ T We make an assumption that the preimage to each extrinsic hash in each work-i...

### 14.4. Computation of Work-Report

We now come to the work-report computation function Ξ. This forms the basis for all utilization of cores on Jam. It accepts some work-package p for some nominated core c and results in either an error ∇ or the work-report and series of exported segments. This function is deterministic and requires only that it be evaluated within eight epochs of a recently finalized block thanks to the historical lookup functionality. It can thus comfortably be evaluated by any node within the auditing period, even allowing for practicalities of imperfect synchronization. Formally: (14.11) Ξ ∶ { (P, N C) → W, (p, c) ↦ { ∇ if o ∉ Y ∶ W R, { s, x ▸ ▸ p x, c, a ▸ ▸ p a, o, l, r, g } otherwise } } Where: K (l) ≡ { h S w ∈ p w, (h ⊞, n) ∈ w i }, S l S ≤ 8 (o, g) = Ψ I (p, c) (r, e) = T [(C (p w [ j ], r, u), e) S (r, u, e) = I (p, j), j ← N S p w S ] I (p, j) ≡ { (⊖, u, [ G 0, G 0 ,. .. ] ...w e) if S r S + z < W R, (⊚, u, [ G 0, G 0 ,. .. ] ...w e) otherwise if S e S ≠ w e, (r, u, [ G 0, G 0 ,. .. ] ...w e) otherwise if r ∉ Y, (r, u, e) otherwise where (r, e, u) = Ψ R (j, p, o, S (w), ℓ) and h = H (p), w = p w [ j ], ℓ = ∑ k < j p w [ k ] e } Note that we gracefully handle both the case where the output size of the work output would take the work-report beyond its acceptable size and where number of segments exported by a work-item's Refinement execution is incorrectly reported in the work-item's export segment count. In both cases, the work-package continues to be valid as a whole, but the work-item's exported segments are replaced by a sequence of zero-segments equal in size to the export segment count and its output is replaced by an error. Initially we constrain the segment-root dictionary l : It should contain entries for all unique work-package hashes of imported segments not identified dire...

---

## 15. Guaranteeing

Guaranteeing work-packages involves the creation and distribution of a corresponding work-report which requires certain conditions to be met. Along with the report, a signature demonstrating the validator's commitment to its correctness is needed. With two guarantor signatures, the work-report may be distributed to the forthcoming Jam chain block author in order to be used in the E G, which leads to a reward for the guarantors. We presume that in a public system, validators will be punished severely if they malfunction and commit to a report which does not faithfully represent the result of Ξ applied on a work-package. Overall, the process is: (1) Evaluation of the work-package's authorization, and cross-referencing against the authorization pool in the most recent Jam chain state. (2) Creation and publication of a work-package report. (3) Chunking of the work-package and each of its extrinsic and exported data, according to the erasure codec. (4) Distributing the aforementioned chunks across the validator set. (5) Providing the work-package, extrinsic and exported data to other validators on request is also helpful for optimal network performance. For any work-package p we are in receipt of, we may determine the work-report, if any, it corresponds to for the core c that we are assigned to. When Jam chain state is needed, we always utilize the chain state of the most recent block. For any guarantor of index v assigned to core c and a work-package p, we define the work-report r simply as: (15.1) r = Ξ (p, c) Such guarantors may safely create and distribute the payload (s, v). The component s may be created according to equation 11.26; specifically it is a signature using the validator's registered Ed25519 key on a payload l : (15.2) l = H (E (c, r)) To maximize profit, the guarantor should require the work-digest meets all expectations which are in place during the guarantee extrinsic descr...

---

## 16. Availability Assurance

Validators should issue a signed statement, called an assurance, when they are in possession of all of their corresponding erasure-coded chunks for a given work-report which is currently pending availability. For any work-report to gain an assurance, there are two classes of data a validator must have: Firstly, their erasure-coded chunk for this report's bundle. The validity of this chunk can be trivially proven through the work-report's work-package erasure-root and a Merkle-proof of inclusion in the correct location. The proof should be included from the guarantor. This chunk is needed to verify the work-report's validity and completeness and need not be retained after the work-report is considered audited. Until then, it should be provided on request to validators. Secondly, the validator should have in hand the corresponding erasure-coded chunk for each of the exported segments referenced by the segments root. These should be retained for 28 days and provided to any validator on request.

---

## 17. Auditing and Judging

The auditing and judging system is theoretically equivalent to that in Elves, introduced by Jeff Burdges, Cevallos, et al. 2024. For a full security analysis of the mechanism, see this work. There is a difference in terminology, where the terms backing, approval and inclusion there refer to our guaranteeing, auditing and accumulation, respectively.

### 17.1. Overview

The auditing process involves each node requiring themselves to fetch, evaluate and issue judgment on a random but deterministic set of work-reports from each Jam chain block in which the work-report becomes available (i.e. from W). Prior to any evaluation, a node declares and proves its requirement. At specific common junctures in time thereafter, the set of work-reports which a node requires itself to evaluate from each block's W may be enlarged if any declared intentions are not matched by a positive judgment in a reasonable time or in the event of a negative judgment being seen. These enlargement events are called tranches. If all declared intentions for a work-report are matched by a positive judgment at any given juncture, then the work-report is considered audited. Once all of any given block's newly available work-reports are audited, then we consider the block to be audited. One prerequisite of a node finalizing a block is for it to view the block as audited. Note that while there will be eventual consensus on whether a block is audited, there may not be consensus at the time that the block gets finalized. This does not affect the crypto-economic guarantees of this system. In regular operation, no negative judgments will ultimately be found for a work-report, and there will be no direct consequences of the auditing stage. In the unlikely event that a negative judgment is found, then one of several things happens; if there are still more than 2 ~ 3 V positive judgments, then validators issuing negative judgments may receive a punishment for time-wasting. If there are greater than 1 ~ 3 V negative judgments, then the block which includes the work-report is ban-listed. It and all its descendants are disregarded and may not be built on. In all cases, once there are enough votes, a judgment extrinsic can be constructed by a block author and placed on-chain to denote the outcome. See section 10 for details on this. All announcements and judgments ...

### 17.2. Data Fetching

For each work-report to be audited, we use its erasure-root to request erasure-coded chunks from enough assurers. From each assurer we fetch three items (which with a good network protocol should be done under a single request) corresponding to the work-package super-chunks, the self-justifying imports super-chunks and the extrinsic segments super-chunks. We may validate the work-package reconstruction by ensuring its hash is equivalent to the hash includes as part of the work-package specification in the work-report. We may validate the extrinsic segments through ensuring their hashes are each equivalent to those found in the relevant work-item. Finally, we may validate each imported segment as a justification must follow the concatenated segments which allows verification that each segment's hash is included in the referencing Merkle root and index of the corresponding work-item. Exported segments need not be reconstructed in the same way, but rather should be determined in the same manner as with guaranteeing, i.e. through the execution of the Refine logic. All items in the work-package specification field of the work-report should be recalculated from this now known-good data and verified, essentially retracing the guarantors steps and ensuring correctness.

### 17.3. Selection of Reports

Each validator shall perform auditing duties on each valid block received. Since we are entering off-chain logic, and we cannot assume consensus, we henceforth consider ourselves a specific validator of index v and assume ourselves focused on some recent block B with other terms corresponding to the state-transition implied by that block, so ρ is said block's prior core-allocation, κ is its prior validator set, H is its header &c. Practically, all considerations must be replicated for all blocks and multiple blocks' considerations may be underway simultaneously. We define the sequence of work-reports which we may be required to audit as Q, a sequence of length equal to the number of cores, which functions as a mapping of core index to a work-report pending which has just become available, or ∅ if no report became available on the core. Formally: Q ∈ ⟦ W ? ⟧ C (17.1) Q ≡ ρ [ c ] w if ρ [ c ] w ∈ W, ∅ otherwise ∀ c ← N C (17.2) We define our initial audit tranche in terms of a verifiable random quantity s 0 created specifically for it: s 0 ∈ F [] κ [ v ] b ⟨ X U ⌢ Y (H v)⟩ (17.3) X U = $jam_audit (17.4) We may then define a 0 as the non-empty items to audit through a verifiably random selection of ten cores: a 0 = { { c, w } S { c, w } ∈ p ⋅⋅⋅+ 10, w ≠ ∅ } (17.5) where p = F ([ { c, Q c } S c ← N C ], r) (17.6) and r = Y (s 0) (17.7) Every A = 8 seconds following a new time slot, a new tranche begins, and we may determine that additional cores warrant an audit from us. Such items are defined as a n where n is the current tranche. Formally: (17.8) let n = T − P ⋅ H t A New tranches may contain items from Q stemming from one of two reasons: either a negative judgment has been received; or the number of judgments from the previous tranche is less than the number of announcements from said tranche. In the first case, the validator is always required to issue a judgment on the work-report. In the second case, a new special-...

---

## 18. Beefy Distribution

For each finalized block B which a validator imports, said validator shall make a bls signature on the bls 12 - 381 curve, as defined by Hopwood et al. 2020, affirming the Keccak hash of the block's most recent Beefy mmr. This should be published and distributed freely, along with the signed material. These signatures may be aggregated in order to provide concise proofs of finality to third-party systems. The signing and aggregation mechanism is defined fully by Jeff Burdges, Ciobotaru, et al. 2022. Formally, let F v be the signed commitment of validator index v which will be published: F v ≡ S κ ′ v (X B ⌢ H K (E M (last (β) b ])) (18.1) X B = $jam_beefy (18.2)

---

## 19. Grandpa and the Best Chain

Nodes take part in the Grandpa protocol as defined by Stewart and Kokoris-Kogia 2020. We define the latest finalized block as B ♮. All associated terms concerning block and state are similarly superscripted. We consider the best block, B ♭ to be that which is drawn from the set of acceptable blocks of the following criteria: Has the finalized block as an ancestor. Contains no unfinalized blocks where we see an equivocation (two valid blocks at the same timeslot). Is considered audited. Formally: A (H ♭) ∋ H ♮ (19.1) U ♭ ≡ ⊺ (19.2) ∄ H A, H B ∶ ⋀ { H A ≠ H B, H A T = H B T, H A ∈ A (H ♭), H A ∉ A (H ♮) } (19.3) Of these acceptable blocks, that which contains the most ancestor blocks whose author used a seal-key ticket, rather than a fallback key should be selected as the best head, and thus the chain on which the participant should make Grandpa votes. Formally, we aim to select B ♭ to maximize the value m where: (19.4) m = ∑ H A ∈ A ♭ T A The specific data to be voted on in Grandpa shall be the block header of the best block, B ♭ together with its posterior state root, M ω (ω ′). The state root has no direct relevance to the Grandpa protocol, but is included alongside the header during voting/signing into order to ensure that systems utilizing the output of Grandpa are able to verify the most recent chain state as possible. This implies that the posterior state must be known at the time that Grandpa voting occurs in order to finalize the block. However, since Grandpa is relied on primarily for state-root verification it makes little sense to finalize a block without an associated commitment to the posterior state. The posterior state only affects Grandpa voting in so much as votes for the same block hash but with different associated posterior state roots are considered votes for different blocks. This would only happen in the case ...

---

## 20. Discussion

### 20.1. Technical Characteristics

In total, with our stated target of 1,023 validators and three validators per core, along with requiring a mean of ten audits per validator per timeslot, and thus 30 audits per work-report, Jam is capable of trustlessly processing and integrating 341 work-packages per timeslot. We assume node hardware is a modern 16 core cpu with 64 gb ram, 8 tb secondary storage and 0.5 g be networking. Our performance models assume a rough split of cpu time as follows:

| Proportion | Activity |
|------------|----------|
| 10 ~ 16 | Audits |
| 1 ~ 16 | Merklization |
| 2 ~ 16 | Block execution |
| 1 ~ 16 | Grandpa and Beefy |
| 1 ~ 16 | Erasure coding |
| 1 ~ 16 | Networking & misc |

Estimates for network bandwidth requirements are as follows:

| Activity | Throughput, mb/slot Tx | Throughput, mb/slot Rx |
|----------|----------------------|----------------------|
| Guaranteeing | 106 | 48 |
| Assuring | 144 | 13 |
| Auditing | 0 | 133 |
| Authoring | 53 | 87 |
| Grandpa and Beefy | 4 | 4 |
| Total | 304 | 281 |
| Implied bandwidth, m b/s | 387 | 357 |

Thus, a connection able to sustain 500 m b/s should leave a sufficient margin of error and headroom to serve other validators as well as some public connections, though the burstiness of block publication would imply validators are best to ensure that peak bandwidth is higher. Under these conditions, we would expect an overall network-provided data availability capacity of 2 pb, with each node dedicating at most 6 tb to availability storage.

Estimates for memory usage are as follows:

| Component | gb |
|-----------|-----|
| Auditing | 20 (2 × 10 pvm instances) |
| Block execution | 2 (1 pvm instance) |
| State cache | 40 |
| Misc | 2 |
| Total | 64 |

As a rough guide, each parachain has an average footprint of around 2 mb in the Polkadot Relay chain; a 40 gb state would allow 20,000 parachains' information to be retained in state. What might be called the "virtual hardware" of a Jam core is essentially a regular cpu core executing at somewhere between 25% and 50% of regular speed for the whole six-second portion and which may draw and provide 2 mb /s average in general-purpose i/o and utilize up to 2 gb in ram. The i/o includes any trustless reads from the Jam chain state, albeit in the recen...

### 20.2. Illustrating Performance

In terms of pure processing power, the Jam machine architecture can deliver extremely high levels of homogeneous trustless computation. However, the core model of Jam is a classic parallelized compute architecture, and for solutions to be able to utilize the architecture well they must be designed with it in mind to some extent. Accordingly, until such use-cases appear on Jam with similar semantics to existing ones, it is very difficult to make direct comparisons to existing systems. That said, if we indulge ourselves with some assumptions then we can make some crude comparisons.

#### 20.2.1. Comparison to Polkadot

Polkadot is at present capable of validating at most 80 parachains each doing one second of native computation and 5 mb of i/o in every six. This corresponds to an aggregate compute performance of around 13x native cpu and a total 24-hour distributed availability of around 67 mb /s. Accumulation is beyond Polkadot's capabilities and so not comparable. For comparison, in our basic models, Jam should be capable of attaining around 85x the computation load of a single native cpu core and a distributed availability of 682 mb /s.

#### 20.2.2. Simple Transfers

We might also attempt to model a simple transactions-per-second amount, with each transaction requiring a signature verification and the modification of two account balances. Once again, until there are clear designs for precisely how this would work we must make some assumptions. Our most naive model would be to use the Jam cores (i.e. refinement) simply for transaction verification and account lookups. The Jam chain would then hold and alter the balances in its state. This is unlikely to give great performance since almost all the needed i/o would be synchronous, but it can serve as a basis. A 12 mb work-package can hold around 96k transactions at 128 bytes per transaction. However, a 48 kb work-result could only encode a...

---

## 21. Conclusion

We have introduced a novel computation model which is able to make use of pre-existing crypto-economic mechanisms in order to deliver major improvements in scalability without causing persistent state-fragmentation and thus sacrificing overall cohesion. We call this overall pattern collect-refine-join-accumulate. Furthermore, we have formally defined the on-chain portion of this logic, essentially the join-accumulate portion. We call this protocol the Jam chain. We argue that the model of Jam provides a novel "sweet spot", allowing for massive amounts of computation to be done in secure, resilient consensus compared to fully-synchronous models, and yet still have strict guarantees about both timing and integration of the computation into some singleton state machine unlike persistently fragmented models.

### 21.1. Further Work

While we are able to estimate theoretical computation possible given some basic assumptions and even make broad comparisons to existing systems, practical numbers are invaluable. We believe the model warrants further empirical research in order to better understand how these theoretical limits translate into real-world performance. We feel a proper cost analysis and comparison to pre-existing protocols would also be an excellent topic for further work. We can be reasonably confident that the design of Jam allows it to host a service under which Polkadot parachains could be validated, however further prototyping work is needed to understand the possible throughput which a pvm-powered metering system could support. We leave such a report as further work. Likewise, we have also intentionally omitted details of higher-level protocol elements including cryptocurrency, coretime sales, staking and regular smart-contract functionality. A number of potential alterations to the protocol described here are being considered in order to make practical utilization of the protocol easier. These include: Synchronous calls between services in accumulate. Restrictions on the transfer function in order to allow for substantial parallelism over accumulation. The possibility of reserving substantial additional computation capacity during accumulate under certain conditions. Introducing Merklization into the Work Package format in order to obviate the need to have the whole package downloaded in order to evaluate its authorization. The networking protocol is also left intentionally undefined at this stage and its description must be done in a follow-up proposal. Validator performance is not presently tracked on-chain. We do expect this to be tracked on-chain in the final revision of the Jam protocol, but its specific format is not yet certain and it is therefore omitted at present.

---

## 22. Acknowledgements

Much of this present work is based in large part on the work of others. The Web3 Foundation research team and in particular Alistair Stewart and Jeff Burdges are responsible for Elves, the security apparatus of Polkadot which enables the possibility of in-core computation for Jam. The same team is responsible for Sassafras, Grandpa and Beefy. Safrole is a mild simplification of Sassafras and was made under the careful review of Davide Galassi and Alistair Stewart. The original CoreJam rfc was refined under the review of Bastian Köcher and Robert Habermeier and most of the key elements of that proposal have made their way into the present work. The pvm is a formalization of a partially simplified PolkaVM software prototype, developed by Jan Bujak. Cyrill Leutwiler contributed to the empirical analysis of the pvm reported in the present work. The PolkaJam team and in particular Arkadiy Paronyan, Emeric Chevalier and Dave Emett have been instrumental in the design of the lower-level aspects of the Jam protocol, especially concerning Merklization and i/o. Numerous contributors to the repository since publication have helped correct errors. Thank you to all. And, of course, thanks to the awesome Lemon Jelly, a.k.a. Fred Deakin and Nick Franglen, for three of the most beautiful albums ever produced, the cover art of the first of which was inspiration for this paper's background art.

---

## Appendix A. Polkadot Virtual Machine

### A.1. Basic Definition

We declare the general pvm function Ψ. We assume a single-step invocation function define Ψ 1 and define the full pvm recursively as a sequence of such mutations up until the single-step mutation results in a halting condition. We additionally define the function deblob which extracts the instruction data, opcode bitmask and dynamic jump table from a program blob: Ψ ∶ { (Y, N R, N G, ⟦ N R ⟧ 13, M) → ({ ∎, ☇, ∞ } ∪ { F, h̵ } × N R, N R, Z G, ⟦ N R ⟧ 13, M), (p, ı, ϱ, ω, μ) ↦ { Ψ (p, ı ′, ϱ ′, ω ′, μ ′) if ε = ▸, (∞, ı ′, ϱ ′, ω ′, μ ′) if ϱ ′ < 0, (ε, 0, ϱ ′, ω ′, μ ′) if ε ∈ { ☇, ∎ }, (ε, ı ′, ϱ ′, ω ′, μ ′) otherwise where (ε, ı ′, ϱ ′, ω ′, μ ′) = { Ψ 1 (c, k, j, ı, ϱ, ω, μ) if { c, k, j } = deblob (p), (☇, ı, ϱ, ω, μ) otherwise } } } (A.1) deblob ∶ { Y → { Y, B, ⟦ N R ⟧ } ∪ ∇, p ↦ { { c, k, j } if ∃ ! c, k, j ∶ p = E (S j S) ⌢ E 1 (z) ⌢ E (S c S) ⌢ E z (j) ⌢ E (c) ⌢ E (k), S k S = S c S, ∇ otherwise } } (A.2) The pvm exit reason ε ∈ { ∎, ☇, ∞ } ∪ { F, h̵ } × N R may be one of regular halt ∎, panic ☇ or out-of-gas ∞, or alternatively a host-call h̵, in which the host-call identifier is associated, or page-fault F in which case the address into ram is associated.

### A.2. Instructions, Opcodes and Skip-distance

The program blob p is split into a series of octets which make up the instruction data c and the opcode bitmask k as well as the dynamic jump table, j. The former two imply an instruction sequence, and by extension a basic-block sequence, itself a sequence of indices of the instructions which follow a block-termination instruction. The latter, dynamic jump table, is a sequence of indices into the instruction data blob and is indexed into when dynamically-computed jumps are taken. It is encoded as a sequence of natural numbers (i.e. non-negative integers) each encoded with the same length in octets. This length, term z above, is itself encoded prior. The pvm counts instructions in octet terms (rather than in terms of instructions) and it is thus convenient to define which octets represent the beginning of an instruction, i.e. the opcode octet, and which do not. This is the purpose of k, the instruction-opcode bitmask. We assert that the length of the bitmask is equal to the length of the instruction blob. We define the Skip function skip which provides the number of octets, minus one, to the next instruction's opcode, given the index of instruction's opcode index into c (and by extension k): (A.3) skip ∶ N → N, i ↦ min (24, j ∈ N ∶ (k ⌢ [ 1, 1 ,. .. ]) i + 1 + j = 1) The Skip function appends k with a sequence of set bits in order to ensure a well-defined result for the final instruction skip (S c S − 1). Given some instruction-index i, its opcode is readily expressed as c i and the distance in octets to move forward to the next instruction is 1 + skip (i). However, each instruction's "length" (defined as the number of contiguous octets starting with the opcode which are needed to fully define the instruction's semantics) is left implicit though limited to being at most 16. We define ζ as being equivalent to the instructions c except with an indefinite sequence of zeroes suffixed to ensure that no out-of-bounds access ...

### A.3. Basic Blocks and Termination Instructions

Instructions of the following opcodes are considered basic-block termination instructions; other than trap & fallthrough, they correspond to instructions which may define the instruction-counter to be something other than its prior value plus the instruction's skip amount: Trap and fallthrough: trap, fallthrough. Jumps: jump, jump_ind. Load-and-Jumps: load_imm_jump, load_imm_jump_ind. Branches: branch_eq, branch_ne, branch_ge_u, branch_ge_s, branch_lt_u, branch_lt_s, branch_eq_imm, branch_ne_imm. Immediate branches: branch_lt_u_imm, branch_lt_s_imm, branch_le_u_imm, branch_le_s_imm, branch_ge_u_imm, branch_ge_s_imm, branch_gt_u_imm, branch_gt_s_imm. We denote this set, as opcode indices rather than names, as T. We define the instruction opcode indices denoting the beginning of basic-blocks as ϖ : (A.5) ϖ ≡ [ 0 ] ⌢ [ n + 1 + skip (n) S n ← N S c S ∧ k n = 1 ∧ c n ∈ T ]

### A.4. Single-Step State Transition

We must now define the single-step pvm state-transition function Ψ 1 : (A.6) Ψ 1 ∶ { (Y, B, ⟦ N R ⟧, N R, N G, ⟦ N R ⟧ 13, M) → ({ ☇, ∎, ▸ } ∪ { F, h̵ } × N R, N R, Z G, ⟦ N R ⟧ 13, M), (c, k, j, ı, ϱ, ω, μ) ↦ (ε, ı ′, ϱ ′, ω ′, μ ′) } We define ε together with the posterior values (denoted as prime) of each of the items of the machine state as being in accordance with the table below. In general, when transitioning machine state for an instruction a number of conditions hold true and instructions are defined essentially by their exceptions to these rules. Specifically, the machine does not halt, the instruction counter increments by one, the gas remaining is reduced by the amount corresponding to the instruction type and ram & registers are unchanged. Formally: (A.7) ε = ▸, ı ′ = ı + 1 + skip (ı), ϱ ′ = ϱ − ϱ ∆, ω ′ = ω, μ ′ = μ except as indicated During the course of executing instructions, ram may be accessed. When an index of ram below 2 16 is required, the machine always panics immediately without further changes to its state regardless of the apparent (in)accessibility of the value. Otherwise, should the given index of ram not be accessible then machine state remains unchanged and the exit reason is a fault with the lowest inaccessible page address to be read. Similarly, where ram must be mutated and yet mutable access is not possible, then machine state is unchanged, and the exit reason is a fault with the lowest page address to be written which is inaccessible. Formally, let r and w be the set of indices by which μ must be subscripted for inspection and mutation respectively in order to calculate the result of Ψ 1. We define the memory-access exceptional execution state ε μ which shall, if not ▸, singly effect the returned return of Ψ 1 as following: let x = { x ∶ x ∈ r ∧ x mod 2 32 ∉ V μ ∨ x ∈ w ∧ x mod 2 32 ∉ V ∗ μ } (A.8) ε μ = { ▸ if x = {}, ☇ if min (x) mod 2 32 < 2 16, F × Z P min (x...

### A.5. Instruction Tables

Note that in the case that the opcode is not defined in the following tables then the instruction is considered invalid, and it results in a panic; ε = ☇. We assume the skip length ℓ is well-defined: (A.19) ℓ ≡ skip (ı)

#### A.5.1. Instructions without Arguments

| ζ ı | Name | ϱ ∆ | Mutations |
|-----|------|-----|-----------|
| 0 | trap | 0 | ε = ☇ |
| 1 | fallthrough | 0 | |

#### A.5.2. Instructions with Arguments of One Immediate

(A.20) let l X = min (4, ℓ), ν X ≡ X l X (E − 1 l X (ζ ı + 1 ⋅⋅⋅+ l X))

| ζ ı | Name | ϱ ∆ | Mutations |
|-----|------|-----|-----------|
| 10 | ecalli | 0 | ε = h̵ × ν X |

#### A.5.3. Instructions with Arguments of One Register and One Extended Width Immediate

(A.21) let r A = min (12, ζ ı + 1 mod 16), ω ′ A ≡ ω ′ r A, ν X ≡ E − 1 8 (ζ ı + 2 ⋅⋅⋅+ 8)

| ζ ı | Name | ϱ ∆ | Mutations |
|-----|------|-----|-----------|
| 20 | load_imm_64 | 0 | ω ′ A = ν X |

(Additional instruction tables continue...)

### A.6. Host Call Definition

An extended version of the pvm invocation which is able to progress an inner host-call state-machine in the case of a host-call halt condition is defined as Ψ H ...

### A.7. Standard Program Initialization

The software programs which will run in each of the four instances where the pvm is utilized in the main document have a very typical setup pattern characteristic of an output of a compiler and linker. This means that ram has sections for program-specific read-only data, read-write (heap) data and the stack. An adjunct to this, very typical of our usage patterns is an extra read-only section via which invocation-specific data may be passed (i.e. arguments). It thus makes sense to define this properly in a single initializer function. These sections are quantized into major zones, and one major zone is always left unallocated between sections in order to reduce accidental overrun. Sections are padded with zeroes to the nearest pvm memory page boundary. We thus define the standard program code format p, which includes not only the instructions and jump table (previously represented by the term c), but also information on the state of the ram at program start. Given program blob p and argument data a, we can decode the program code c, registers ω, and ram μ by invoking the standard initialization function Y (p, a) ...

### A.8. Argument Invocation Definition

The four instances where the pvm is utilized each expect to be able to pass argument data in and receive some return data back. We thus define the common pvm program-argument invocation function Ψ M ...

---

## Appendix B. Virtual Machine Invocations

We now define the four practical instances where we wish to invoke a PVM instance as part of the protocol. In general we avoid introducing unbounded data as part of the basic invocation arguments in order to minimise the chance of an unexpectedly large RAM allocation, which could lead to gas inflation and unavoidable underflow. This makes for a more cumbersome interface, but one which is more predictable and easier to reason about.

### B.1. Host-Call Result Constants

- NONE = 2 64 − 1 : The return value indicating an item does not exist.
- WHAT = 2 64 − 2 : Name unknown.
- OOB = 2 64 − 3 : The inner pvm memory index provided for reading/writing is not accessible.
- WHO = 2 64 − 4 : Index unknown.
- FULL = 2 64 − 5 : Storage full.
- CORE = 2 64 − 6 : Core index unknown.
- CASH = 2 64 − 7 : Insufficient funds.
- LOW = 2 64 − 8 : Gas limit too low.
- HUH = 2 64 − 9 : The item is already solicited or cannot be forgotten.
- OK = 0 : The return value indicating general success.

Inner pvm invocations have their own set of result codes:
- HALT = 0 : The invocation completed and halted normally.
- PANIC = 1 : The invocation completed with a panic.
- FAULT = 2 : The invocation completed with a page fault.
- HOST = 3 : The invocation completed with a host-call fault.
- OOG = 4 : The invocation completed by running out of gas.

Note return codes for a host-call-request exit are any non-zero value less than 2 64 − 13.

### B.2. Is-Authorized Invocation

The Is-Authorized invocation is the first and simplest of the four, being totally stateless. It provides only host-call functions for inspecting its environment and parameters. It accepts as arguments only the core on which it should be executed, c. Formally, it is defined as Ψ I ...

### B.3. Refine Invocation

We define the Refine service-account invocation function as Ψ R. It has no general access to the state of the Jam chain, with the slight exception being the ability to make a historical lookup. Beyond this it is able to create inner instances of the pvm and dictate pieces of data to export. The historical-lookup host-call function, Ω H, is designed to give the same result regardless of the state of the chain for any time when auditing may occur (which we bound to be less than two epochs from being accumulated). The lookup anchor may be up to L timeslots before the recent history and therefore adds to the potential age at the time of audit. We therefore set D to have a safety margin of eight hours: (B.3) D ≡ L + 4, 800 = 19, 200 ...

### B.4. Accumulate Invocation

Since this is a transition which can directly affect a substantial amount of on-chain state, our invocation context is accordingly complex. It is a tuple with elements for each of the aspects of state which can be altered through this invocation and beyond the account of the service itself includes the deferred transfer list and several dictionaries for alterations to preimage lookup state, core assignments, validator key assignments, newly created accounts and alterations to account privilege levels...

### B.5. On-Transfer Invocation

We define the On-Transfer service-account invocation function as Ψ T ; it is somewhat similar to the Accumulation Invocation except that the only state alteration it facilitates are basic alteration to the storage of the subject account. No further transfers may be made, no privileged operations are possible, no new accounts may be created nor other operations done on the subject account itself...

### B.6. General Functions

We come now to defining the host functions which are utilized by the pvm invocations. Generally, these map some pvm state, including invocation context, possibly together with some additional parameters, to a new pvm state...

### B.7. Refine Functions

These assume some refine context pair (m, e) ∈ (D ⟨ N → M ⟩, ⟦ G ⟧), which are both initially empty. Other than the gas-counter which is explicitly defined, elements of pvm state are each assumed to remain unchanged by the host-call unless explicitly specified...

### B.8. Accumulate Functions

This defines a number of functions broadly of the form (ϱ ′ ∈ Z G, ω ′ ∈ ⟦ N R ⟧ 13, μ ′, (x ′, y ′)) = Ω ◻ (ϱ ∈ N G, ω ∈ ⟦ N R ⟧ 13, μ ∈ M, (x ∈ X, y ∈ X) ,. ..). Functions which have a result component which is equivalent to the corresponding argument may have said components elided in the description. Functions may also depend upon particular additional parameters...

---

## Appendix C. Serialization Codec

### C.1. Common Terms

Our codec function E is used to serialize some term into a sequence of octets. We define the deserialization function E − 1 as the inverse of E and able to decode some sequence into the original value. The codec is designed such that exactly one value is encoded into any given sequence of octets, and in cases where this is not desirable then we use special codec functions.

#### C.1.1. Trivial Encodings

We define the serialization of ∅ as the empty sequence: (C.1) E (∅) ≡ [] We also define the serialization of an octet-sequence as itself: (C.2) E (x ∈ Y) ≡ x We define anonymous tuples to be encoded as the concatenation of their encoded elements: (C.3) E ({ a, b,. .. }) ≡ E (a) ⌢ E (b) ⌢. .. Passing multiple arguments to the serialization functions is equivalent to passing a tuple of those arguments. Formally: E (a, b,. ..) ≡ E ({ a, b,. .. }) (C.4)

#### C.1.2. Integer Encoding

We first define the trivial natural number serialization functions which are subscripted by the number of octets of the final sequence. Values are encoded in a regular little-endian fashion. This is utilized for almost all integer encoding across the protocol...

#### C.1.3. Sequence Encoding

We define the sequence serialization function E (⟦ T ⟧) for any T which is itself a subset of the domain of E. We simply concatenate the serializations of each element in the sequence in turn...

### C.2. Block Serialization

A block B is serialized as a tuple of its elements in regular order, as implied in equations 4.2, 4.3 and 5.1. For the header, we define both the regular serialization and the unsigned serialization E U...

---

## Appendix D. State Merklization

The Merklization process defines a cryptographic commitment from which arbitrary information within state may be provided as being authentic in a concise and swift fashion. We describe this in two stages; the first defines a mapping from 31-octet sequences to (unlimited) octet sequences in a process called state serialization. The second forms a 32-octet commitment from this mapping in a process called Merklization.

### D.1. Serialization

The serialization of state primarily involves placing all the various components of σ into a single mapping from 31-octet sequence state-keys to octet sequences of indefinite length. The state-key is constructed from a hash component and a chapter component, equivalent to either the index of a state component or, in the case of the inner dictionaries of δ, a service index...

### D.2. Merklization

With T defined, we now define the rest of M σ which primarily involves transforming the serialized mapping into a cryptographic commitment. We define this commitment as the root of the binary Patricia Merkle Trie with a format optimized for modern compute hardware, primarily by optimizing sizes to fit succinctly into typical memory layouts and reducing the need for unpredictable branching.

#### D.2.1. Node Encoding and Trie Identification

We identify (sub-)tries as the hash of their root node, with one exception: empty (sub-)tries are identified as the zero-hash, H 0. Nodes are fixed in size at 512 bit (64 bytes). Each node is either a branch or a leaf. The first bit discriminate between these two types...

---

## Appendix E. General Merklization

### E.1. Binary Merkle Trees

The Merkle tree is a cryptographic data structure yielding a hash commitment to a specific sequence of values. It provides O (N) computation and O (log (N)) proof size for inclusion. This well-balanced formulation ensures that the maximum depth of any leaf is minimal and that the number of leaves at that depth is also minimal...

#### E.1.1. Well-Balanced Tree

We define the well-balanced binary Merk...

### E.2. Merkle Mountain Ranges

The Merkle Mountain Range (mmr) is an append-only cryptographic data structure which yields a commitment to a sequence of values. Appending to an mmr and proof of inclusion of some item within it are both O (log (N)) in time and space for the size of the set. We define a Merkle Mountain Range as being within the set ⟦ H ? ⟧, a sequence of peaks, each peak the root of a Merkle tree containing 2 i items where i is the index in the sequence...

---

## Appendix F. Shuffling

The Fisher-Yates shuffle function is defined formally as: (F.1) ∀ T, l ∈ N ∶ F ∶ { (⟦ T ⟧ l, ⟦ N ⟧ l ∶) → ⟦ T ⟧ l, (s, r) ↦ { [ s r 0 mod l ] ⌢ F (s ′ ...l − 1, r 1 ...) where s ′ = s except s ′ r 0 mod l = s l − 1 if s ≠ [], [] otherwise } } Since it is often useful to shuffle a sequence based on some random seed in the form of a hash, we provide a secondary form of the shuffle function F which accepts a 32-byte hash instead of the numeric sequence. We define Q, the numeric-sequence-from-hash function, thus: ∀ l ∈ N ∶ Q l ∶ { H → ⟦ N 2 32 ⟧ l, h ↦ [ E − 1 4 (H (h ⌢ E 4 (⌊ i ~ 8 ⌋)) 4 i mod 32 ⋅⋅⋅+ 4) S i ← N l ] } (F.2) ∀ T, l ∈ N ∶ F ∶ (⟦ T ⟧ l, H) → ⟦ T ⟧ l, (s, h) ↦ F (s, Q l (h)) (F.3)

---

## Appendix G. Bandersnatch VRF

The Bandersnatch curve is defined by Masson, Sanso, and Zhang 2021. The singly-contextualized Bandersnatch Schnorr-like signatures F m k ⟨ c ⟩ are defined as a formulation under the IETF vrf template specified by Hosseini and Galassi 2024 (as IETF VRF) and further detailed by Goldberg et al. 2023. F m ∈ Y k ∈ H B ⟨ c ∈ H ⟩ ⊂ Y 96 ≡ { x S x ∈ Y 96, verify (k, c, m, x) = ⊺ } (G.1) Y (s ∈ F m k ⟨ c ⟩) ∈ H ≡ output (x S x ∈ F m k ⟨ c ⟩) ... 32 (G.2) The singly-contextualized Bandersnatch Ring vrf proofs F̄ m r ⟨ c ⟩ are a zksnark-enabled analogue utilizing the Pedersen vrf, also defined by Hosseini and Galassi 2024 and further detailed by Jeffrey Burdges et al. 2023. O (⟦ H B ⟧) ∈ Y R ≡ commit (⟦ H B ⟧) (G.3) F̄ m ∈ Y r ∈ Y R ⟨ c ∈ H ⟩ ⊂ Y 784 ≡ { x S x ∈ Y 784, verify (r, c, m, x) = ⊺ } (G.4) Y (p ∈ F̄ m r ⟨ c ⟩) ∈ H ≡ output (x S x ∈ F̄ m r ⟨ c ⟩) ... 32 (G.5) Note that in the case a key H B has no corresponding Bandersnatch point when constructing the ring, then the Bandersnatch padding point as stated by Hosseini and Galassi 2024 should be substituted.

---

## Appendix H. Erasure Coding

The foundation of the data-availability and distribution system of Jam is a systematic Reed-Solomon erasure coding function in gf (2 16) of rate 342:1023, the same transform as done by the algorithm of Lin, Chung, and Han 2014. We use a little-endian Y 2 form of the 16-bit gf points with a functional equivalence given by E 2. From this we may assume the encoding function C ∶ ⟦ Y 2 ⟧ 342 → ⟦ Y 2 ⟧ 1023 and the recovery function R ∶ ℘ ⟨ { Y 2, N 1023 } ⟩ 342 → ⟦ Y 2 ⟧ 342. Encoding is done by extrapolating a data blob of size 684 octets (provided in C here as 342 octet pairs) into 1,023 octet pairs. Recovery is done by collecting together any distinct 342 octet pairs, together with their indices, and transforming this into the original sequence of 342 octet pairs. Practically speaking, this allows for the efficient encoding and recovery of data whose size is a multiple of 684 octets. Data whose length is not divisible by 684 must be padded (we pad with zeroes). We use this erasure-coding in two contexts within the Jam protocol; one where we encode variable sized (but typically very large) data blobs for the Audit da and block-distribution system, and the other where we encode much smaller fixed-size data segments for the Import da system.

### H.1. Blob Encoding and Recovery

We assume some data blob d ∈ Y 684 k, k ∈ N. We are able to express this as a whole number of k pieces each of a sequence of 684 octets. We denote these (data-parallel) pieces p ∈ ⟦ Y 684 ⟧ = unzip 684 (d). Each piece is then reformed as 342 octet pairs and erasure-coded using C as above to give 1,023 octet pairs per piece. The resulting matrix is grouped by its pair-index and concatenated to form 1,023 chunks, each of k octet-pairs. Any 342 of these chunks may then be used to reconstruct the original data d...

### H.2. Code Word representation

For the sake of brevity we call each octet pair a word. The code words (including the message words) are treated as element of F 2 16 finite field. The field is generated as an extension of F 2 using the irreducible polynomial: (H.8) x 16 + x 5 + x 3 + x 2 + 1 Hence: (H.9) F 16 ≡ F 2 [ x ] x 16 + x 5 + x 3 + x 2 + 1 We name the generator of F 16 F 2, the root of the above polynomial, α as such: F 16 = F 2 (α). Instead of using the standard basis { 1, α, α 2 ,. .., α 15 }, we opt for a representation of F 16 which performs more efficiently for the encoding and the decoding process...

### H.3. The Generator Polynomial

To erasure code a message of 342 words into 1023 code words, we represent each message as a field element as described in previous section and we interpolate the polynomial p (y) of maximum 341 degree which satisfies the following equalities: (H.12) p (˜ 0) = È m 0, p (˜ 1) = È m 1, ⋮, p (É 341) = Ê m 341 After finding p (y) with such properties, we evaluate p at the following points: (H.13) É r 342 ∶ = p (É 342), É r 343 ∶ = p (É 343), ⋮, Ê r 1022 ∶ = p (Ê 1022) We then distribute the message words and the extra code words among the validators according to their corresponding indices.

---

## Appendix I. Index of Notation

### I.1. Sets

#### I.1.1. Regular Notation

- N : The set of non-negative integers. Subscript denotes one greater than the maximum. See section 3.4.
- N + : The set of positive integers (not including zero).
- N B : The set of balance values. Equivalent to N 2 64. See equation 4.21.
- N G : The set of unsigned gas values. Equivalent to N 2 64. See equation 4.23.
- N L : The set of blob length values. Equivalent to N 2 32. See section 3.4.
- N R : The set of register values. Equivalent to N 2 64. See equation 4.23.
- N S : The set from which service indices are drawn. Equivalent to N 2 32. See section 9.1.
- N T : The set of timeslot values. Equivalent to N 2 32. See equation 4.28.
- Q : The set of rational numbers. Unused.
- R : The set of real numbers. Unused.
- Z : The set of integers. Subscript denotes range. See section 3.4.
- Z G : The set of signed gas values. Equivalent to Z − 2 63 ... 2 63. See equation 4.23.

#### I.1.2. Custom Notation

- A : The set of service accounts. See equation 9.3.
- B : The set of Boolean sequences/bitstrings. Subscript denotes length. See section 3.7.
- C : The set of seal-key tickets. See equation 6.6. Not used as the set of complex numbers.
- D : The set of dictionaries. See section 3.5.
- D ⟨ K → V ⟩ : The set of dictionaries making a partial bijection of domain K to range V. See section 3.5.
- E : The set of valid Ed25519 signatures. A subset of Y 64. See section 3.8.
- E K ⟨ M ⟩ : The set of valid Ed25519 signatures of the key K and message M. A subset of E. See section 3.8.
- F : The set of Bandersnatch signatures. A subset of Y 64. See section 3.8.
- F M K ⟨ C ⟩ : The set of Bandersnatch signatures of the public key K, context C and message M. A subset of F. See section 3.8.
- F̄ : The set of Bandersnatch Ring vrf proofs. See section 3.8.
- F̄ M R ⟨ C ⟩ : The set of Bandersnatch Ring vrf proofs of the root R, context C and message M. A subset of F̄. See section 3.8.
- G : The set of data segments, equivalent to octet sequences of length W G. See e...

### I.2. Functions

- ∆ : The accumulation function; certain subscripts are used to denote helper functions...
- Λ : The historical lookup function. See equation 9.7.
- Ξ : The work-digest computation function. See equation 14.11.
- Υ : The general state transition function. See equations 4.1, 4.5.
- Φ : The key-nullifier function. See equation 6.14.
- Ψ : The whole-program pvm machine state-transition function. See equation A.
- Ψ 1 : The single-step (pvm) machine state-transition function. See appendix A.
- Ψ A : The Accumulate pvm invocation function. See appendix B.
- Ψ H : The host-function invocation (pvm) with host-function marshalling. See appendix A.
- Ψ I : The Is-Authorized pvm invocation function. See appendix B.
- Ψ M : The marshalling whole-program pvm machine state-transition function. See appendix A.
- Ψ R : The Refine pvm invocation function. See appendix B.
- Ψ T : The On-Transfer pvm invocation function. See appendix B.
- Ω : Virtual machine host-call functions. See appendix B.
- Ω A : Assign-core host-call.
- Ω B : Empower-service host-call.
- Ω C : Checkpoint host-call.
- Ω D : Designate-validators host-call.
- Ω E : Export segment host-call.
- Ω F : Forget-preimage host-call.
- Ω G : Gas-remaining host-call.
- Ω H : Historical-lookup-preimage host-call.
- Ω I : Information-on-service host-call.
- Ω J : Eject-service host-call.
- Ω K : Kickoff-pvm host-call.
- Ω L : Lookup-preimage host-call.
- Ω M : Make-pvm host-call.
- Ω N : New-service host-call.
- Ω O : Poke-pvm host-call.
- Ω P : Peek-pvm host-call.
- Ω Q : Query-preimage host-call.
- Ω R : Read-storage host-call.
- Ω S : Solicit-preimage host-call.
- Ω T : Transfer host-call.
- Ω U : Upgrade-service host-call.
- Ω V : Void inner-pvm memory host-call.
- Ω W : Write-storage host-call.
- Ω X : Expunge-pvm host-call.
- Ω Y : Fetch data host-call.
- Ω Z : Zero inner-pvm memory host-call.

### I.3. Utilities, Externalities and Standard Functions

- A (. ..) : The Merkle mountain range append function. See equation E.8.
- B n (. ..) : The octets-to-bits function for n octets. Superscripted − 1 to denote the inverse. See equation A.12.
- C (. ..) : The group of erasure-coding functions.
- C n (. ..) : The erasure-coding functions for n chunks. See equation H.6.
- E (. ..) : The octet-sequence encode function. Superscripted − 1 to denote the inverse. See appendix C.
- F (. ..) : The Fisher-Yates shuffle function. See equation F.1.
- H (. ..) : The Blake 2b 256-bit hash function. See section 3.8.
- H K (. ..) : The Keccak 256-bit hash function. See section 3.8.
- J x : The justification path to a specific 2 x size page of a constant-depth Merkle tree. See equation E.5.
- K (. ..) : The domain, or set of keys, of a dictionary. See section 3.5.
- L x : The 2 x size page function for a constant-depth Merkle tree. See equation E.6.
- M (. ..) : The constant-depth binary Merklization function. See appendix E.
- M B (. ..) : The well-balanced binary Merklization function. See appendix E.
- M σ (. ..) : The state Merklization function. See appendix D.
- N (. ..) : The erasure-coding chunks function. See appendix H.
- O (. ..) : The Bandersnatch ring root function. See section 3.8 and appendix G.
- P n (. ..) : The octet-array zero-padding function. See equation 14.17.
- Q (. ..) : The numeric-sequence-from-hash function. See equation F.3.
- R : The group of erasure-coding piece-recovery functions.
- S k (. ..) : The general signature function. See section 3.8.
- T : The current time expressed in seconds after the start of the Jam Common Era. See section 4.4.
- U (. ..) : The substitute-if-nothing function. See equation 3.2.
- V (. ..) : The range, or set of values, of a dictionary or sequence. See section 3.5.
- X n (. ..) : The signed-extension function for a value in N 2 8 n. See equation A.16.
- Y (. ..) : The alias/output/entropy function of a Bandersn...

### I.4. Values

#### I.4.1. Block-context Terms

These terms are all contextualized to a single block. They may be superscripted with some other term to alter the context and reference some other block.

- A : The ancestor set of the block. See equation 5.3.
- B : The block. See equation 4.2.
- C : The service accumulation-commitment, used to form the Beefy root. See equation ??.
- E : The block extrinsic. See equation 4.3.
- F v : The Beefy signed commitment of validator v. See equation 18.1.
- G : The mapping from cores to guarantor keys. See section 11.3.
- G ∗ : The mapping from cores to guarantor keys for the previous rotation. See section 11.3.
- H : The block header. See equation 5.1.
- I : The sequence of work-reports which were accumulated this in this block. See equation ??.
- Q : The selection of ready work-reports which a validator determined they must audit. See equation 17.1.
- R : The set of Ed25519 guarantor keys who made a work-report. See equation 11.26.
- S : The set of indices of services which have been accumulated ("progressed") in the block. See equation ??.
- T : The ticketed condition, true if the block was sealed with a ticket signature rather than a fallback. See equations 6.15 and 6.16.
- U : The audit condition, equal to ⊺ once the block is audited. See section 17.
- V : The set of verdicts in the present block. See equation 10.11.
- W : The sequence of work-reports which have now become available and ready for accumulation. See equation 11.16.
- X : The sequence of transfers implied by the block's accumulations. See equation 12.30.

Without any superscript, the block is assumed to the block being imported or, if no block is being imported, the head of the best chain (see section 19). Explicit block-contextualizing superscripts include:
- B ♮ : The latest finalized block. See equation 19.
- B ♭ : The block at the head of the best chain. See equation 19.

#### I.4.2. State components

Here, the prime annotation indicates posterior state. Individual components may be identified with a lette...

---

## References

Bertoni, Guido et al. (2013). "Keccak". In: Annual international conference on the theory and applications of cryptographic techniques. Springer, pp. 313–314. 

Bögli, Roman (2024). "Assessing risc Zero using ZKit: An Extensible Testing and Benchmarking Suite for ZKP Frameworks". PhD thesis. OST Ostschweizer Fachhochschule. 

Boneh, Dan, Ben Lynn, and Hovav Shacham (2004). "Short Signatures from the Weil Pairing". In: J. Cryptology 17, pp. 297–319. doi: 10.1007/s00145-004-0314-9. 

Burdges, Jeff, Alfonso Cevallos, et al. (2024). Efficient Execution Auditing for Blockchains under Byzantine Assumptions. Cryptology ePrint Archive, Paper 2024/961. https://eprint.iacr.org/2024/961. url: https://eprint.iacr.org/2024/961. 

Burdges, Jeff, Oana Ciobotaru, et al. (2022). Efficient Aggregatable BLS Signatures with Chaum-Pedersen Proofs. Cryptology ePrint Archive, Paper 2022/1611. https://eprint.iacr.org/2022/1611. url: https://eprint.iacr.org/2022/1611. 

Burdges, Jeffrey et al. (2023). Ring Verifiable Random Functions and Zero-Knowledge Continuations. Cryptology ePrint Archive, Paper 2023/002. url: https://eprint.iacr.org/2023/002. 

Buterin, Vitalik (2013). Ethereum: A Next-Generation Smart Contract and Decentralized Application Platform. url: https://github.com/ethereum/wiki/wiki/White-Paper. 

Buterin, Vitalik and Virgil Griffith (2019). Casper the Friendly Finality Gadget. arXiv: 1710.09437 [cs.CR]. 

Cosmos Project (2023). Interchain Security Begins a New Era for Cosmos. Fetched 18th March, 2024. url: https://blog.cosmos.network/interchain-security-begins-a-new-era-for-cosmos-a2dc3c0be63. 

Dune and hildobby (2024). Ethereum Staking. Fetched 18th March, 2024. url: https://dune.com/hildobby/eth2staking. 

Ethereum Foundation (2024a). "A digital future on a global scale". In: Fetched 4th April, 2024. url: https://ethereum.org/en/roadmap/vision/. 

Ethereum Foundation (2024b). Danksharding. Fetched 18th March, 2024. url: https://ethereum.org/en/roadmap/danksharding/. 

Fisher, Ronald Ayl...
