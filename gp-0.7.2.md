

## JOIN-ACCUMULATE MACHINE: A MOSTLY-COHERENT TRUSTLESS SUPERCOMPUTER
DRAFT 0.7.2 - September 15, 2025
## DR. GAVIN WOOD
## FOUNDER, POLKADOT & ETHEREUM
## GAVIN@PARITY.IO
Abstract.We present a comprehensive and formal definition of
## J
am, a protocol combining elements of bothPolkadot
andEthereum. In a single coherent model,
## J
amprovides a global singleton permissionless object environment—much
like the smart-contract environment pioneered by Ethereum—paired with secure sideband computation parallelized
over a scalable node network, a proposition pioneered by Polkadot.
## J
amintroduces a decentralized hybrid system offering smart-contract functionality structured around a secure and
scalable in-core/on-chain dualism. While the smart-contract functionality implies some similarities with Ethereum’s
paradigm, the overall model of the service offered is driven largely by underlying architecture of Polkadot.
## J
amis permissionless in nature, allowing anyone to deploy code as a service on it for a fee commensurate with the
resources this code utilizes and to induce execution of this code through the procurement and allocation ofcore-time,
a metric of resilient and ubiquitous computation, somewhat similar to the purchasing of gas in Ethereum. We already
envision a Polkadot-compatibleCoreChainsservice.
1.Introduction
1.1.Nomenclature.In this paper, we introduce a de-
centralized, crypto-economic protocol to which the Polka-
dot Network will transition itself in a major revision on
the basis of approval by its governance apparatus.
An early, unrefined, version of this protocol was
first proposed in Polkadot Fellowshiprfc31, known
asCoreJam.  CoreJam takes its name after the col-
lect/refine/join/accumulate model of computation at the
heart of its service proposition. While the CoreJamrfc
suggested an incomplete, scope-limited alteration to the
Polkadot protocol,
## J
amrefers to a complete and coherent
overall blockchain protocol.
1.2.Driving Factors.Within the realm of blockchain
and the wider Web3, we are driven by the need first and
foremost to deliver resilience. A proper Web3 digital sys-
tem should honor a declared service profile—and ideally
meet even perceived expectations—regardless of the de-
sires, wealth or power of any economic actors including in-
dividuals, organizations and, indeed, other Web3 systems.
Inevitably this is aspirational, and we must be pragmatic
over how perfectly this may really be delivered. Nonethe-
less, a Web3 system should aim to provide such radically
strong guarantees that, for practical purposes, the system
may be described asunstoppable.
While Bitcoin is, perhaps, the first example of such a
system within the economic domain, it was not general
purpose in terms of the nature of the service it offered. A
rules-based service is only as useful as the generality of the
rules which may be conceived and placed within it. Bit-
coin’s rules allowed for an initial use-case, namely a fixed-
issuance token, ownership of which is well-approximated
and autonomously enforced through knowledge of a secret,
as well as some further elaborations on this theme.
Later, Ethereum would provide a categorically more
general-purpose rule set, one which was practically Tur-
ing complete.
## 1
In the context of Web3 where we are aim-
ing to deliver a massively multiuser application platform,
generality is crucial, and thus we take this as a given.
Beyond resilience and generality, things get more in-
teresting, and we must look a little deeper to understand
what our driving factors are. For the present purposes,
we identify three additional goals:
(1)Resilience: highly resistant from being stopped,
corrupted and censored.
(2)Generality:  able to perform Turing-complete
computation.
## 1
The gas mechanism did restrict what programs can execute on it by placing an upper bound on the number of steps which may be
executed, but some restriction to avoid infinite-computation must surely be introduced in a permissionless setting.
## 1

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 20252
(3)Performance:  able to perform computation
quickly and at low cost.
(4)Coherency: the causal relationship possible be-
tween different elements of state and thus how
well individual applications may be composed.
(5)Accessibility: negligible barriers to innovation;
easy, fast, cheap and permissionless.
As a declared Web3 technology, we make an implicit
assumption of the first two items. Interestingly, items3
and4are antagonistic according to an information the-
oretic principle which we are sure must already exist in
some form but are nonetheless unaware of a name for it.
For argument’s sake we shall name itsize-coherency an-
tagonism.
1.3.Scaling under Size-Coherency Antagonism.
Size-coherency antagonism is a simple principle implying
that as the state-space of information systems grow, then
the system necessarily becomes less coherent. It is a direct
implication of principle that causality is limited by speed.
The maximum speed allowed by physics isCthe speed
of light in a vacuum, however other information systems
may have lower bounds: In biological system this is largely
determined by various chemical processes whereas in elec-
tronic systems is it determined by the speed of electrons
in various substances. Distributed software systems will
tend to have much lower bounds still, being dependent
on a substrate of software, hardware and packet-switched
networks of varying reliability.
The argument goes:
(1)The more state a system utilizes for its data-
processing, the greater the amount of space this
state must occupy.
(2)The more space used, then the greater the
mean and variance of distances between state-
components.
(3)As the mean and variance increase, then time for
causal resolution (i.e. all correct implications of
an event to be felt) becomes divergent across the
system, causing incoherence.
Setting the question of overall security aside for a mo-
ment, we can manage incoherence by fragmenting the sys-
tem into causally-independent subsystems, each of which
is small enough to be coherent. In a resource-rich en-
vironment, a bacterium may split into two rather than
growing to double its size. This pattern is rather a crude
means of dealing with incoherency under growth: intra-
system processing has low size and total coherence, inter-
system processing supports higher overall sizes but with-
out coherence. It is the principle behind meta-networks
such as Polkadot, Cosmos and the predominant vision of
a scaled Ethereum (all to be discussed in depth shortly).
Such systems typically rely on asynchronous and simplis-
tic communication with “settlement areas” which provide
a small-scoped coherent state-space to manage specific in-
teractions such as a token transfer.
The present work explores a middle-ground in the an-
tagonism, avoiding the persistent fragmentation of state-
space of the system as with existing approaches. We do
this by introducing a new model of computation which
pipelines a highly scalable,mostly coherentelement to a
synchronous, fully coherent element. Asynchrony is not
avoided, but we bound it to the length of the pipeline and
substitute the crude partitioning we see in scalable sys-
tems so far with a form of “cache aﬀinity” as it typically
seen in multi-cpusystems with a sharedram.
Unlike withsnark-based L2-blockchain techniques for
scaling, this model draws upon crypto-economic mecha-
nisms and inherits their low-cost and high-performance
profiles and averts a bias toward centralization.
1.4.Document Structure.We begin with a brief
overview of present scaling approaches in blockchain tech-
nology in section2. In section3we define and clarify the
notation from which we will draw for our formalisms.
We follow with a broad overview of the protocol in sec-
tion4outlining the major areas including the Polkadot
Virtual Machine (pvm), the consensus protocols Safrole
andGrandpa, the common clock and build the founda-
tions of the formalism.
We then continue with the full protocol definition split
into two parts: firstly the correct on-chain state-transition
formula helpful for all nodes wishing to validate the chain
state, and secondly, in sections14and19the honest strat-
egy for the off-chain actions of any actors who wield a
validator key.
The main body ends with a discussion over the per-
formance characteristics of the protocol in section
## 20and
finally conclude in section21.
The appendix contains various additional material im-
portant for the protocol definition including thepvmin
appendicesA&B, serialization and Merklization in ap-
pendicesC&Dand cryptography in appendicesE,G&
H. We finish with an index of terms which includes the
values of all simple constant terms used in the work in
appendixI, and close with the bibliography.
2.Previous Work and Present Trends
In the years since the initial publication of the
EthereumYP, the field of blockchain development has
grown immensely. Other than scalability, development
has been done around underlying consensus algorithms,
smart-contract languages and machines and overall state
environments. While interesting, these latter subjects are
mostly out scope of the present work since they generally
do not impact underlying scalability.
2.1.Polkadot.In order to deliver its service,
## J
amco-
opts much of the same game-theoretic and cryptographic
machinery as Polkadot known asElvesand described by
Jeff Burdges, Cevallos, et al.2024. However, major differ-
ences exist in the actual service offered with
## J
am, provid-
ing an abstraction much closer to the actual computation
model generated by the validator nodes its economy in-
centivizes.
It was a major point of the original Polkadot pro-
posal, a scalable heterogeneous multichain, to deliver high-
performance through partition and distribution of the
workload over multiple host machines. In doing so it took
an explicit position that composability would be lowered.
Polkadot’s constituent components, parachains are, prac-
tically speaking, highly isolated in their nature. Though a
message passing system (xcmp) exists it is asynchronous,
coarse-grained and practically limited by its reliance on a
high-level slowly evolving interaction languagexcm.
As such, the composability offered by Polkadot be-
tween its constituent chains is lower than that of

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 20253
Ethereum-like smart-contract systems offering a single
and universal object environment and allowing for the
kind of agile and innovative integration which underpins
their success. Polkadot, as it stands, is a collection of
independent ecosystems with only limited opportunity
for collaboration, very similar in ergonomics to bridged
blockchains though with a categorically different security
profile. A technical proposal known asspreewould uti-
lize Polkadot’s unique shared-security and improve com-
posability, though blockchains would still remain isolated.
Implementing and launching a blockchain is hard, time-
consuming and costly. By its original design, Polkadot
limits the clients able to utilize its service to those who
are both able to do this and raise a suﬀicient deposit to
win an auction for a long-term slot, one of around 50 at
the present time. While not permissioned per se, acces-
sibility is categorically and substantially lower than for
smart-contract systems similar to Ethereum.
Enabling as many innovators to participate and inter-
act, both with each other and each other’s user-base, ap-
pears to be an important component of success for a Web3
application platform. Accessibility is therefore crucial.
2.2.Ethereum.The Ethereum protocol was formally de-
fined in this paper’s spiritual predecessor, theYellow Pa-
per, by Wood2014. This was derived in large part from
the initial concept paper by Buterin2013. In the decade
since theYPwas published, thede factoEthereum proto-
col and public network instance have gone through a num-
ber of evolutions, primarily structured around introducing
flexibility via the transaction format and the instruction
set and “precompiles” (niche, sophisticated bonus instruc-
tions) of its scripting core, the Ethereum virtual machine
## (evm).
Almost one million crypto-economic actors take part
in the validation for Ethereum.
## 2
Block extension is done
through a randomized leader-rotation method where the
physical address of the leader is public in advance of their
block production.
## 3
Ethereum uses Casper-ffgintroduced
by Buterin and Griﬀith
2019to determine finality, which
with the large validator base finalizes the chain extension
around every 13 minutes.
Ethereum’s direct computational performance remains
broadly similar to that with which it launched in 2015,
with a notable exception that an additional service now
allows 1mbofcommitment datato be hosted per block
(all nodes to store it for a limited period). The data can-
not be directly utilized by the main state-transition func-
tion, but special functions provide proof that the data
(or some subsection thereof) is available. According to
## Ethereum Foundation
2024b, the present design direction
is to improve on this over the coming years by splitting
responsibility for its storage amongst the validator base in
a protocol known asDank-sharding.
According to Ethereum Foundation
2024a, the scaling
strategy of Ethereum would be to couple this data avail-
ability with a private market ofroll-ups, sideband com-
putation facilities of various design, withzk-snark-based
roll-ups being a stated preference. Each vendor’s roll-up
design, execution and operation comes with its own impli-
cations.
One might reasonably assume that a diversified market-
based approach for scaling via multivendor roll-ups will al-
low well-designed solutions to thrive. However, there are
potential issues facing the strategy. A research report by
Sharma2023on the level of decentralization in the vari-
ous roll-ups found a broad pattern of centralization, but
notes that work is underway to attempt to mitigate this.
It remains to be seen how decentralized they can yet be
made.
Heterogeneous communication properties (such as
datagram latency and semantic range), security properties
(such as the costs for reversion, corruption, stalling and
censorship) and economic properties (the cost of accept-
ing and processing some incoming message or transaction)
may differ, potentially quite dramatically, between major
areas of some grand patchwork of roll-ups by various com-
peting vendors. While the overall Ethereum network may
eventually provide some or even most of the underlying
machinery needed to do the sideband computation it is
far from clear that there would be a “grand consolidation”
of the various properties should such a thing happen. We
have not found any good discussion of the negative rami-
fications of such a fragmented approach.
## 4
2.2.1.SnarkRoll-ups.While the protocol’s foundation
makes no great presuppositions on the nature of roll-ups,
Ethereum’s strategy for sideband computation does cen-
tre aroundsnark-based rollups and as such the protocol
is being evolved into a design that makes sense for this.
Snarks are the product of an area of exotic cryptography
which allow proofs to be constructed to demonstrate to a
neutral observer that the purported result of performing
some predefined computation is correct. The complexity
of the verification of these proofs tends to be sub-linear in
their size of computation to be proven and will not give
away any of the internals of said computation, nor any
dependent witness data on which it may rely.
Zk-snarks come with constraints. There is a trade-off
between the proof’s size, verification complexity and the
computational complexity of generating it. Non-trivial
computation, and especially the sort of general-purpose
computation laden with binary manipulation which makes
smart-contracts so appealing, is hard to fit into the model
ofsnarks.
To give a practical example,risc-zero (as assessed by
## Bögli
2024) is a leading project and provides a platform
for producingsnarks of computation done by arisc-v
virtual machine, an open-source and succinctriscma-
chine architecture well-supported by tooling. A recent
benchmarking report by PolkavmProject
## 2024showed
that compared torisc-zero’s own benchmark, proof gen-
eration alone takes over 61,000 times as long as simply re-
compiling and executing even when executing on 32 times
as many cores, using 20,000 times as muchramand an
additional state-of-the-artgpu. According to hardware
## 2
Practical matters do limit the level of real decentralization. Validator software expressly provides functionality to allow a single instance
to be configured with multiple key sets, systematically facilitating a much lower level of actual decentralization than the apparent number
of actors, both in terms of individual operators and hardware. Using data collated by Dune and hildobby
2024on Ethereum 2, one can see
one major node operator, Lido, has steadily accounted for almost one-third of the almost one million crypto-economic participants.
## 3
Ethereum’s developers hope to change this to something more secure, but no timeline is fixed.
## 4
Some initial thoughts on the matter resulted in a proposal by Sadana2024to utilize Polkadot technology as a means of helping create
a modicum of compatibility between roll-up ecosystems!

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 20254
rental agentshttps://cloud-gpus.com/, the cost multi-
plier of proving usingrisc-zero is 66,000,000x of the cost
## 5
to execute using the Polkavmrecompiler.
Many cryptographic primitives become too expensive
to be practical to use and specialized algorithms and struc-
tures must be substituted. Often times they are otherwise
suboptimal. In expectation of the use ofsnarks (such as
plonkas proposed by Gabizon, Williamson, and Ciobo-
taru2019), the prevailing design of the Ethereum project’s
Dank-sharding availability system uses a form of erasure
coding centered around polynomial commitments over a
large prime field in order to allowsnarks to get accept-
ably performant access to subsections of data. Compared
to alternatives, such as a binary field and Merklization in
the present work, it leads to a load on the validator nodes
orders of magnitude higher in terms ofcpuusage.
In addition to their basic cost,snarks present no great
escape from decentralization and the need for redundancy,
leading to further cost multiples. While the need for some
benefits of staked decentralization is averted through their
verifiable nature, the need to incentivize multiple parties
to do much the same work is a requirement to ensure that
a single party not form a monopoly (or several not form
a cartel). Proving an incorrect state-transition should be
impossible, however service integrity may be compromised
in other ways; a temporary suspension of proof-generation,
even if only for minutes, could amount to major economic
ramifications for real-time financial applications.
Real-world examples exist of the pit of centralization
giving rise to monopolies. One would be the aforemen-
tionedsnark-based exchange framework; while notionally
serving decentralized exchanges, it is in fact centralized
with Starkware itself wielding a monopoly over enacting
trades through the generation and submission of proofs,
leading to a single point of failure—should Starkware’s ser-
vice become compromised, then the liveness of the system
would suffer.
It has yet to be demonstrated thatsnark-based strate-
gies for eliminating the trust from computation will ever
be able to compete on a cost-basis with a multi-party
crypto-economic platform. All as-yet proposedsnark-
based solutions are heavily reliant on crypto-economic sys-
tems to frame them and work around their issues. Data
availability and sequencing are two areas well understood
as requiring a crypto-economic solution.
We would note thatsnarktechnology is improving
and the cryptographers and engineers behind them do ex-
pect improvements in the coming years. In a recent arti-
cle by Thaler
2023we see some credible speculation that
with some recent advancements in cryptographic tech-
niques, slowdowns for proof generation could be as lit-
tle as 50,000x from regular native execution and much
of this could be parallelized. This is substantially bet-
ter than the present situation, but still several orders of
magnitude greater than would be required to compete on
a cost-basis with established crypto-economic techniques
such asElves.
2.3.Fragmented  Meta-Networks.Directions  for
general-purpose computation scalability taken by other
projects broadly centre around one of two approaches;
either what might be termed afragmentationapproach
or alternatively acentralizationapproach. We argue that
neither approach offers a compelling solution.
The fragmentation approach is heralded by projects
such as Cosmos (proposed by Kwon and Buchman2019)
and Avalanche (by Tanana2019). It involves a system
fragmented by networks of a homogenous consensus me-
chanic, yet staffed by separately motivated sets of valida-
tors. This is in contrast to Polkadot’s single validator set
and Ethereum’s declared strategy of heterogeneous roll-
ups secured partially by the same validator set operating
under a coherent incentive framework. The homogeneity
of said fragmentation approach allows for reasonably con-
sistent messaging mechanics, helping to present a fairly
unified interface to the multitude of connected networks.
However, the apparent consistency is superficial. The
networks are trustless only by assuming correct operation
of their validators, who operate under a crypto-economic
security framework ultimately conjured and enforced by
economic incentives and punishments. To do twice as
much work with the same levels of security and no special
coordination between validator sets, then such systems es-
sentially prescribe forming a new network with the same
overall levels of incentivization.
Several problems arise.  Firstly, there is a simi-
lar downside as with Polkadot’s isolated parachains and
Ethereum’s isolated roll-up chains: a lack of coherency
due to a persistently sharded state preventing synchro-
nous composability.
More problematically, the scaling-by-fragmentation
approach, proposed specifically by Cosmos, provides
no homogenous security—and therefore trustlessness—
guarantees.  Validator sets between networks must be
assumed to be independently selected and incentivized
with no relationship, causal or probabilistic, between the
Byzantine actions of a party on one network and potential
for appropriate repercussions on another. Essentially, this
means that should validators conspire to corrupt or revert
the state of one network, the effects may be felt across
other networks of the ecosystem.
That this is an issue is broadly accepted, and projects
propose for it to be addressed in one of two ways. Firstly,
to fix the expected cost-of-attack (and thus level of se-
curity) across networks by drawing from the same val-
idator set. The massively redundant way of doing this,
as proposed by Cosmos Project
2023under the name
replicated security, would be to require each validator
to validate on all networks and for the same incentives
and punishments. This is economically ineﬀicient in the
cost of security provision as each network would need to
independently provide the same level of incentives and
punishment-requirements as the most secure with which
it wanted to interoperate. This is to ensure the economic
proposition remain unchanged for validators and the se-
curity proposition remained equivalent for all networks.
At the present time, replicated security is not a readily
available permissionless service. We might speculate that
these punishing economics have something to do with it.
The more eﬀicient approach, proposed by the Om-
niLedger team, Kokoris-Kogias et al.
2017, would be to
## 5
In all likelihood actually substantially more as this was using low-tier “spare” hardware in consumer units, and our recompiler was
unoptimized.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 20255
make the validators non-redundant, partitioning them be-
tween different networks and periodically, securely and
randomly repartitioning them. A reduction in the cost
to attack over having them all validate on a single net-
work is implied since there is a chance of having a single
network accidentally have a compromising number of ma-
licious validators even with less than this proportion over-
all. This aside it presents an effective means of scaling
under a basis of weak-coherency.
Alternatively, as inElvesby Jeff Burdges, Cevallos,
et al.2024, we may utilize non-redundant partitioning,
combine this with a proposal-and-auditing game which
validators play to weed out and punish invalid computa-
tions, and then require that the finality of one network
be contingent on all causally-entangled networks. This
is the most secure and economically eﬀicient solution of
the three, since there is a mechanism for being highly
confident that invalid transitions will be recognized and
corrected before their effect is finalized across the ecosys-
tem of networks. However, it requires substantially more
sophisticated logic and their causal-entanglement implies
some upper limit on the number of networks which may
be added.
2.4.High-Performance Fully Synchronous Net-
works.Another trend in the recent years of blockchain
development has been to make “tactical” optimizations
over data throughput by limiting the validator set size or
diversity, focusing on software optimizations, requiring a
higher degree of coherency between validators, onerous re-
quirements on the hardware which validators must have,
or limiting data availability.
The Solana blockchain is underpinned by technology
introduced by Yakovenko2018and boasts theoretical fig-
ures of over 700,000 transactions per second, though ac-
cording to Ng
2024the network is only seen processing a
small fraction of this. The underlying throughput is still
substantially more than most blockchain networks and is
owed to various engineering optimizations in favor of max-
imizing synchronous performance. The result is a highly-
coherent smart-contract environment with anapinot un-
like that ofYPEthereum (albeit using a different under-
lyingvm), but with a near-instant time to inclusion and
finality which is taken to be immediate upon inclusion.
Two issues arise with such an approach: firstly, defin-
ing the protocol as the outcome of a heavily optimized
codebase creates structural centralization and can under-
mine resilience. Jha
## 2024writes “since January 2022, 11
significant outages gave rise to 15 days in which major
or partial outages were experienced”. This is an outlier
within the major blockchains as the vast majority of ma-
jor chains have no downtime. There are various causes to
this downtime, but they are generally due to bugs found
in various subsystems.
Ethereum, at least until recently, provided the most
contrasting alternative with its well-reviewed specifica-
tion, clear research over its crypto-economic foundations
and multiple clean-room implementations.  It is per-
haps no surprise that the network very notably contin-
ued largely unabated when a flaw in its most deployed
implementation was found and maliciously exploited, as
described by Hertig2016.
The second issue is concerning ultimate scalability of
the protocol when it provides no means of distributing
workload beyond the hardware of a single machine.
In major usage, both historical transaction data and
state would grow impractically. Solana illustrates how
much of a problem this can be.   Unlike classical
blockchains, the Solana protocol offers no solution for the
archival and subsequent review of historical data, crucial
if the present state is to be proven correct from first prin-
ciple by a third party. There is little information on how
Solana manages this in the literature, but according to
Solana Foundation2023, nodes simply place the data onto
a centralized database hosted by Google.
## 6
Solana validators are encouraged to install large
amounts oframto help hold its large state in mem-
ory (512gbis the current recommendation according to
Solana Labs2024). Without a divide-and-conquer ap-
proach, Solana shows that the level of hardware which
validators can reasonably be expected to provide dictates
the upper limit on the performance of a totally synchro-
nous, coherent execution model. Hardware requirements
represent barriers to entry for the validator set and cannot
grow without sacrificing decentralization and, ultimately,
transparency.
3.Notational Conventions
Much as in the Ethereum Yellow Paper, a number of
notational conventions are used throughout the present
work. We define them here for clarity. The Ethereum
Yellow Paper itself may be referred to henceforth as the
## YP.
3.1.Typography.We use a number of different type-
faces to denote different kinds of terms. Where a term is
used to refer to a value only relevant within some localized
section of the document, we use a lower-case roman letter
e.g.x,y(typically used for an item of a set or sequence)
or e.g.i,j(typically used for numerical indices). Where
we refer to a Boolean term or a function in a local context,
we tend to use a capitalized roman alphabet letter such as
A,F. If particular emphasis is needed on the fact a term
is sophisticated or multidimensional, then we may use a
bold typeface, especially in the case of sequences and sets.
For items which retain their definition throughout the
present work, we use other typographic conventions. Sets
are usually referred to with a blackboard typeface, e.g.N
refers to all natural numbers including zero. Sets which
may be parameterized may be subscripted or be followed
by parenthesized arguments. Imported functions, used by
the present work but not specifically introduced by it, are
written in calligraphic typeface, e.g.Hthe Blake2 cryp-
tographic hashing function. For other non-context depen-
dent functions introduced in the present work, we use up-
per case Greek letters, e.g.Υdenotes the state transition
function.
Values which are not fixed but nonetheless hold some
consistent meaning throughout the present work are de-
noted with lower case Greek letters such asσ, the state
## 6
Earlier node versions utilized Arweave network, a decentralized data store, but this was found to be unreliable for the data throughput
which Solana required.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 20256
identifier. These may be placed in bold typeface to denote
that they refer to an abnormally complex value.
3.2.Functions and Operators.We define the precedes
relation to indicate that one term is defined in terms of
another. E.g.y≺xindicates thatymay be defined purely
in terms ofx:
y≺x⇐⇒∃f∶y=f(x)(3.1)
The substitute-if-nothing functionUis equivalent to
the first argument which is not∅, or∅if no such argu-
ment exists:
## U(a
## 0
## ,...a
n
## )≡a
x
## ∶(a
x
## ≠∅ ∨x=n),
x−1
## ⋀
i=0
a
i
## =∅(3.2)
Thus, e.g.U(∅,1,∅,2)=1andU(∅,∅)=∅.
3.3.Sets.Given some sets, its power set and cardinal-
ity are denoted as{[s]}andSsS. When forming a power
set, we may use a numeric subscript in order to restrict
the resultant expansion to a particular cardinality. E.g.
## {[ {1,2,3} ]}
## 2
## ={ {1,2},{1,3},{2,3} }.
Sets may be operated on with scalars, in which case
the result is a set with the operation applied to each el-
ement, e.g.{1,2,3}+3={4,5,6}. Functions may also
be applied to all members of a set to yield a new set,
but for clarity we denote this with a#superscript, e.g.
f
## #
## ({1,2})≡{f(1),f(2) }.
We denote set-disjointness with the relation⫰. For-
mally:
## A∩B=∅⇐⇒A⫰B
We commonly use∅to indicate that some term is
validly left without a specific value.  Its cardinality is
defined as zero. We define the operation?such that
A?≡A∪{∅}indicating the same set but with the addi-
tion of the∅element.
The term∇is utilized to indicate the unexpected fail-
ure of an operation or that a value is invalid or unexpected.
(We try to avoid the use of the more conventionalhere
to avoid confusion with Boolean false, which may be in-
terpreted as some successful result in some contexts.)
3.4.Numbers.Ndenotes the set of naturals including
zero whereasN
n
implies a restriction on that set to val-
ues less thann.  Formally,N={0,1,...}andN
n
## =
{xSx∈N,x<n}.
Zdenotes the set of integers. We denoteZ
a...b
to be
the set of integers within the interval[a,b). Formally,
## Z
a...b
={xSx∈Z,a≤x<b}. E.g.Z
## 2...5
## ={2,3,4}. We
denote the offset/length form of this set asZ
a⋅⋅⋅+b
, a short
form ofZ
a...a+b
## .
It can sometimes be useful to represent lengths of se-
quences and yet limit their size, especially when dealing
with sequences of octets which must be stored practically.
Typically, these lengths can be defined as the setN
## 2
## 32
## .
To improve clarity, we denoteN
## L
as the set of lengths of
octet sequences and is equivalent toN
## 2
## 32
## .
We denote the%operator as the modulo operator,
e.g.5%3=2. Furthermore, we may occasionally express
a division result as a quotient and remainder with the
separatorR, e.g.5÷3=1R2.
3.5.Dictionaries.Adictionaryis a possibly partial
mapping from some domain into some co-domain in much
the same manner as a regular function. Unlike functions
however, with dictionaries the total set of pairings are
necessarily enumerable, and we represent them in some
data structure as the set of all(key↦value)pairs. (In
such data-defined mappings, it is common to name the
values within the domain akeyand the values within the
co-domain avalue, hence the naming.)
Thus, we define the formalismjK→Voto denote a dic-
tionary which maps from the domainKto the rangeV.
It is a subset of the power set of pairs
## ⎧
## ⎩
## K,V
## ⎫
## ⎭
## :
(3.3)jK→Vo⊂{[
## ⎧
## ⎩
## K,V
## ⎫
## ⎭
## ]}
The subset is caused by a constraint that a dictionary’s
members must associate at most one unique value for any
given keyk:
(3.4)∀K,V,d∈jK→Vo∶ ∀(k,v)∈d∶ ∃!v
## ′
## ∶
## 
k,v
## ′
## 
## ∈d
In the context of a dictionary we denote the pairs with
a mapping notation:
jK→Vo≡{[
## ⎧
## ⎩
## K→V
## ⎫
## ⎭
## ]}(3.5)
p∈
## ⎧
## ⎩
## K→V
## ⎫
## ⎭
⇔∃k∈K,v∈V,p≡(k↦v)
## (3.6)
This assertion allows us to unambiguously define the
subscript and subtraction operator for a dictionaryd:
∀K,V,d∈jK→Vo∶d[k]≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
vif∃k∶(k↦v)∈d
## ∅otherwise
## (3.7)
∀K,V,d∈jK→Vo,s⊆K∶
d∖s≡{ (k↦v)∶(k↦v)∈d,k~∈s}
## (3.8)
Note that when using a subscript, it is an implicit as-
sertion that the key exists in the dictionary. Should the
key not exist, the result is undefined and any block which
relies on it must be considered invalid.
To denote the active domain (i.e. set of keys) of a dic-
tionaryd∈jK→Vo, we useK(d)⊆Kand for the range
(i.e. set of values),V(d)⊆V. Formally:
∀K,V,d∈jK→Vo∶K(d)≡{kS∃v∶(k↦v)∈d}
## (3.9)
∀K,V,d∈jK→Vo∶V(d)≡{vS∃k∶(k↦v)∈d}(3.10)
Note that since the co-domain ofV()is a set, should
different keys with equal values appear in the dictionary,
the set will only contain one such value.
Dictionaries may be combined through the union oper-
ator∪, which priorities the right-side operand in the case
of a key-collision:
## (3.11)
∀d∈K,V,(d,e)∈jK→Vo
## 2
∶d∪e≡(d∖K(e))∪e
3.6.Tuples.Tuples are groups of values where each item
may belong to a different set. They are denoted with
parentheses, e.g. the tupletof the naturals3and5is de-
notedt=(3,5), and it exists in the set of natural pairs
sometimes denotedN×N, but denoted in the present work
as
## ⎧
## ⎩
## N,N
## ⎫
## ⎭
## .
We have frequent need to refer to a specific item within
a tuple value and as such find it convenient to declare a
name for each item. E.g. we may denote a tuple with two
named natural componentsaandbasT=
## ⎧
## ⎩
a∈N, b∈N
## ⎫
## ⎭
## .
We would denote an itemt∈Tthrough subscripting its
name, thus for somet=(a
## ▸
## ▸
3, b
## ▸
## ▸
## 5),t
a
## =3andt
b
## =5.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 20257
3.7.Sequences.A sequence is a series of elements with
particular ordering not dependent on their values. The set
of sequences of elements all of which are drawn from some
setTis denoted⟦T⟧, and it defines a partial mapping
N→T. The set of sequences containing exactlynele-
ments each a member of the setTmay be denoted⟦T⟧
n
and accordingly defines a complete mappingN
n
→T. Sim-
ilarly, sets of sequences of at mostnelements and at least
nelements may be denoted⟦T⟧
## ∶n
and⟦T⟧
n∶
respectively.
Sequences are subscriptable, thus a specific item at in-
dexiwithin a sequencesmay be denoteds[i], or where
unambiguous,s
i
. A range may be denoted using an ellip-
sis for example:[0,1,2,3]
## ...2
## =[0,1]and[0,1,2,3]
## 1⋅⋅⋅+2
## =
[1,2]. The length of such a sequence may be denotedSsS.
We denote modulo subscription ass[i]
## ↺
≡s[i%SsS ].
We denote the final elementxof a sequences=[...,x]
through the function last(s)≡x.
3.7.1.Construction.We may wish to define a sequence
in terms of incremental subscripts of other values:
## [x
## 0
## ,x
## 1
## ,...]
## ...n
denotes a sequence ofnvalues beginning
x
## 0
continuing up tox
n−1
.  Furthermore, we may also
wish to define a sequence as elements each of which
are a function of their indexi; in this case we denote
## [f(i) Si<
## −N
n
]≡[f(0),f(1),...,f(n−1)]. Thus, when
the ordering of elements matters we use<−rather than
the unordered notation∈. The latter may also be written
in short form[f(i<−N
n
)]. This applies to any set which
has an unambiguous ordering, particularly sequences, thus
## 
i
## 2
## T
i<−[1,2,3]
## 
=[1,4,9]. Multiple sequences may be
combined, thus[i⋅jSi<−[1,2,3],j<−[2,3,4]]=[2,6,12].
As with sets, we use explicit notationf
## #
to denote a
function mapping over all items of a sequence.
Sequences may be constructed from sets or other se-
quences whose order should be ignored through sequence
ordering notation[i∈X
## ^
## ^
f(i)], which is defined to result
in the set or sequence of its argument except that all ele-
mentsiare placed in ascending order of the corresponding
valuef(i).
The key component may be elided in which case it
is assumed to be ordered by the elements directly; i.e.
[i∈X]≡[i∈X
## ^
## ^
i].[i∈X
## _
## _
i]does the same, but excludes
any duplicate values ofi. E.g. assumings=[1,3,2,3],
then[i∈s
## _
## _
i]=[1,2,3]and[i∈s
## ^
## ^
## −i]=[3,3,2,1].
Sets may be constructed from sequences with the reg-
ular set construction syntax, e.g. assumings=[1,2,3,1],
then{aSa∈s}would be equivalent to{1,2,3}.
Sequences of values which themselves have a defined
ordering have an implied ordering akin to a regular dic-
tionary, thus[1,2,3]<[1,2,4]and[1,2,3]<[1,2,3,1].
3.7.2.Editing.We define the sequence concatenation op-
erator⌢such that[x
## 0
## ,x
## 1
## ,...,y
## 0
## ,y
## 1
## ,...]≡x⌢y. For
sequences of sequences, we define a unary concatenate-all
operator:
## Ì
x≡x
## 0
## ⌢x
## 1
⌢.... Further, we denote ele-
ment concatenation asx
i≡x⌢[i]. We denote the
sequence made up of the firstnelements of sequencesto
be
## Ð→
s
n
## ≡[s
## 0
## ,s
## 1
## ,...,s
n−1
], and only the final elements as
## ←Ð
s
n
## .
We define
## T
xas the transposition of the sequence-of-
sequencesx, fully defined in equation
H.3. We may also
apply this to sequences-of-tuples to yield a tuple of se-
quences.
We denote sequence subtraction with a slight modifica-
tion of the set subtraction operator; specifically, some se-
quencesexcepting the left-most element equal tovwould
be denotedsm{v}.
3.7.3.Boolean values.b
s
denotes the set of Boolean
strings of lengths, thusb
s
## =⟦{,⊺}⟧
s
. When dealing
with Boolean values we may assume an implicit equiva-
lence mapping to a bit whereby⊺=1and=0, thus
b
## ◻
## =⟦N
## 2
## ⟧
## ◻
. We use the function bits(B)∈bto de-
note the sequence of bits, ordered with the most signif-
icant first, which represent the octet sequenceB, thus
bits([160,0])=[1,0,1,0,0,...].
The unary-not operator applies to both boolean val-
ues and sequences of boolean values, thus¬⊺=and
## ¬[⊺,]=[,⊺].
3.7.4.Octets and Blobs.Bdenotes the set of octet strings
(“blobs”) of arbitrary length. As might be expected,B
x
denotes the set of such sequences of lengthx.B
## $
denotes
the subset ofBwhich areascii-encoded strings. Note that
while an octet has an implicit and obvious bijective rela-
tionship with natural numbers less than 256, and we may
implicitly coerce between octet form and natural number
form, we do not treat them as exactly equivalent entities.
In particular for the purpose of serialization, an octet is
always serialized to itself, whereas a natural number may
be serialized as a sequence of potentially several octets,
depending on its magnitude and the encoding variant.
## 3.7.5.
## Shuffling.
We define the sequence-shuffle function
F, originally introduced by Fisher and Yates1938, with an
eﬀicient in-place algorithm described by Wikipedia2024.
This accepts a sequence and some entropy and returns a
sequence of the same length with the same elements but
in an order determined by the entropy. The entropy may
be provided as either an indefinite sequence of naturals or
a hash. For a full definition see appendixF.
3.8.Cryptography.
3.8.1.Hashing.Hdenotes the set of 256-bit values equiv-
alent toB
## 32
. All hash functions in the present work out-
put to this type andH
## 0
is the value equal to[0]
## 32
## . We
assume a functionH(m∈B)∈Hdenoting the Blake2b
256-bit hash introduced by Saarinen and Aumasson2015
and a function
## H
## K
## (
m
## ∈
## B
## )
## ∈
## H
denoting the Keccak 256-
bit hash as proposed by Bertoni et al.2013and utilized
by Wood
## 2014.
The inputs of a hash function should be expected to
be passed through our serialization codecEto yield an
octet sequence to which the cryptography may be ap-
plied. (Note that an octet sequence conveniently yields
an identity transform.) We may wish to interpret a se-
quence of octets as some other kind of value with the as-
sumed decoder functionE
## −1
(x∈B). In both cases, we may
subscript the transformation function with the number of
octets we expect the octet sequence term to have. Thus,
r=E
## 4
(x∈N)would assertx∈N
## 2
## 32
andr∈B
## 4
, whereas
s=E
## −1
## 8
(y)would asserty∈B
## 8
ands∈N
## 2
## 64
## .
3.8.2.Signing Schemes.
## ̄
## V
k
⟨m⟩⊂B
## 64
is the set of valid
Ed25519 signatures, defined by Josefsson and Liusvaara
2017, made through knowledge of a secret key whose pub-
lic key counterpart isk∈Hand whose message ism. To
aid readability, we denote the set of valid public keys
## ̄
## H.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 20258
We denote the set of valid Bandersnatch public keys as
## ∽
H, defined in appendixG.
## ∽
## V
m∈B
k∈
## ∽
## H
⟨x∈B⟩⊂B
## 96
is the set of
valid singly-contextualized signatures of utilizing the se-
cret counterpart to the public keyk, some contextxand
messagem.
## ○
## V
m∈B
r∈
## ○
## B
⟨x∈B⟩⊂B
## 784
, meanwhile, is the set of valid Ban-
dersnatch Ringvrfdeterministic singly-contextualized
proofs of knowledge of a secret within some set of secrets
identified by some root in the set of validroots
## ○
## B⊂B
## 144
## .
We denoteOs∈D
## ∽
## HI∈
## ○
Bto be the root specific to the set
of public key counterpartss. A root implies a specific set
of Bandersnatch key pairs, knowledge of one of the secrets
would imply being capable of making a unique, valid—and
anonymous—proof of knowledge of a unique secret within
the set.
Both the Bandersnatch signature and Ringvrfproof
strictly imply that a member utilized their secret key in
combination with both the contextxand the messagem;
the difference is that the member is identified in the for-
mer and is anonymous in the latter. Furthermore, both
define avrfoutput, a high entropy hash influenced by
xbut not bym, formally denotedY
## ○
## V
m
r
⟨x⟩⊂Hand
## Y
## ∽
## V
m
k
⟨x⟩⊂H.
We use
## BLS
## B⊂B
## 144
to denote the set of public keys for
theblssignature scheme, described by Boneh, Lynn, and
## Shacham
2004, on curvebls12-381defined by Hopwood
et al.2020. We correspondingly use the notation
## BLS
## V
k
## ⟨m⟩to
denote the set of validblssignatures for public keyk∈
## BLS
## B
and messagem∈B.
We define the signature functions for creating valid sig-
natures;
## ̄
## S(m)∈
## ̄
## V
k
## ⟨m⟩,
## BLS
## S(m)∈
## BLS
## V
k
⟨m⟩. We assert that
the ability to compute a result for this function relies on
knowledge of a secret key.
4.Overview
As in the Yellow Paper, we begin our formalisms by
recalling that a blockchain may be defined as a pairing
of some initial state together with a block-level state-
transition function. The latter defines the posterior state
given a pairing of some prior state and a block of data
applied to it. Formally, we say:
σ
## ′
≡Υ(σ,B)(4.1)
Whereσis the prior state,σ
## ′
is the posterior state,Bis
some valid block andΥis our block-level state-transition
function.
Broadly speaking,
## J
am(and indeed blockchains in gen-
eral) may be defined simply by specifyingΥand somegen-
esis stateσ
## 0
## .
## 7
We also make several additional assump-
tions of agreed knowledge: a universally known clock, and
the practical means of sharing data with other systems
operating under the same consensus rules. The latter two
were both assumptions silently made in theYP.
4.1.The Block.To aid comprehension and definition of
our protocol, we partition as many of our terms as possible
into their functional components. We begin with the block
Bwhich may be restated as the headerHand some input
data external to the system and thus said to beextrinsic,
## E:
## B≡(H,E)(4.2)
## E≡
## (
## E
## T
## ,E
## D
## ,E
## P
## ,E
## A
## ,E
## G
## )
## (4.3)
The header is a collection of metadata primarily con-
cerned with cryptographic references to the blockchain an-
cestors and the operands and result of the present tran-
sition. As an immutable knowna priori, it is assumed
to be available throughout the functional components of
block transition. The extrinsic data is split into its several
portions:
tickets:Tickets, used for the mechanism which
manages the selection of validators for the per-
missioning of block authoring. This component is
denotedE
## T
## .
preimages:Static data which is presently being re-
quested to be available for workloads to be able
to fetch on demand. This is denotedE
## P
## .
reports:Reports of newly completed workloads
whose accuracy is guaranteed by specific valida-
tors. This is denotedE
## G
## .
availability:Assurances by each validator concern-
ing which of the input data of workloads they have
correctly received and are storing locally. This is
denotedE
## A
## .
disputes:Information relating to disputes between
validators over the validity of reports. This is de-
notedE
## D
## .
4.2.The State.Our state may be logically partitioned
into several largely independent segments which can both
help avoid visual clutter within our protocol description
and provide formality over elements of computation which
may be simultaneously calculated (i.e. parallelized). We
therefore pronounce an equivalence betweenσ(some com-
plete state) and a tuple of partitioned segments of that
state:
σ≡(α,β,θ,γ,δ,η,ι,κ,λ,ρ,τ,φ,χ,ψ,π,ω,ξ)
## (4.4)
In summary,δis the portion of state dealing withser-
vices, analogous in
## J
amto the Yellow Paper’s (smart con-
tract)accounts, the only state of theYP’s Ethereum. The
identities of services which hold some privileged status are
tracked inχ.
Validators, who are the set of economic actors uniquely
privileged to help build and maintain the
## J
amchain, are
identified withinκ, archived inλand enqueued fromι. All
other state concerning the determination of these keys is
held withinγ. Note this is a departure from theYPproof-
of-work definitions which were mostly stateless, and this
set was not enumerated but rather limited to those with
suﬀicient compute power to find a partial hash-collision in
thesha2-256cryptographic hash function. An on-chain
entropy pool is retained inη.
Our state also tracks two aspects of each core:α, the
authorization requirement which work done on that core
must satisfy at the time of being reported on-chain, to-
gether with the queue which fills this,φ; andρ, each of the
cores’ currently assignedreport, the availability of whose
## 7
Practically speaking, blockchains sometimes make assumptions of some fraction of participants whose behavior is simplyhonest, and
not provably incorrect nor otherwise economically disincentivized. While the assumption may be reasonable, it must nevertheless be stated
apart from the rules of state-transition.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 20259
work-packagemust yet be assured by a super-majority of
validators.
Finally, details of the most recent blocks and timeslot
index are tracked inβ
## H
andτrespectively, work-reports
which are ready to be accumulated and work-packages
which were recently accumulated are tracked inωandξ
respectively and, judgments are tracked inψand validator
statistics are tracked inπ.
4.2.1.State Transition Dependency Graph.Much as in
theYP, we specifyΥas the implication of formulating
all items of posterior state in terms of the prior state and
block. To aid the architecting of implementations which
parallelize this computation, we minimize the depth of
the dependency graph where possible. The overall depen-
dency graph is specified here:
τ
## ′
## ≺H(4.5)
β
## †
## H
≺(H,β
## H
## )(4.6)
γ
## ′
## ≺
## 
H,τ,E
## T
## ,γ,ι,η
## ′
## ,κ
## ′
## ,ψ
## ′
## 
## (4.7)
η
## ′
≺(H,τ,η)(4.8)
κ
## ′
≺(H,τ,κ,γ)(4.9)
λ
## ′
≺(H,τ,λ,κ)(4.10)
ψ
## ′
## ≺(E
## D
## ,ψ)(4.11)
ρ
## †
## ≺(E
## D
## ,ρ)(4.12)
ρ
## ‡
## ≺E
## A
## ,ρ
## †
## (4.13)
ρ
## ′
## ≺E
## G
## ,ρ
## ‡
## ,κ,τ
## ′
## (4.14)
## R
## ∗
## ≺E
## A
## ,ρ
## †
## 
## (4.15)
## ω
## ′
## ,ξ
## ′
## ,δ
## ‡
## ,χ
## ′
## ,ι
## ′
## ,φ
## ′
## ,θ
## ′
## ,S≺
## 
## R
## ∗
## ,ω,ξ,δ,χ,ι,φ,τ,τ
## ′
## 
## (4.16)
β
## ′
## H
## ≺H,E
## G
## ,β
## †
## H
## ,θ
## ′
## (4.17)
δ
## ′
## ≺E
## P
## ,δ
## ‡
## ,τ
## ′
## (4.18)
α
## ′
## ≺
## 
## H,E
## G
## ,φ
## ′
## ,α
## 
## (4.19)
π
## ′
## ≺
## 
## E
## G
## ,E
## P
## ,E
## A
## ,E
## T
## ,τ,κ
## ′
,π,H,S
## 
## (4.20)
The only synchronous entanglements are visible
through the intermediate components superscripted with
a dagger and defined in equations4.6,4.12,4.13,4.14,
4.16,4.17and4.18. The latter two mark a merge and
join in the dependency graph and, concretely, imply that
the availability extrinsic may be fully processed and ac-
cumulation of work happen before the preimage lookup
extrinsic is folded into state.
4.3.Which History?A blockchain is a sequence of
blocks, each cryptographically referencing some prior
block by including a hash of its header, all the way back
to some first block which references the genesis header.
We already presume consensus over this genesis header
## H
## 0
and the state it represents already defined asσ
## 0
## .
By defining a deterministic function for deriving a sin-
gle posterior state for any (valid) combination of prior
state and block, we are able to define a uniquecanonical
state for any given block. We generally call the block with
the most ancestors theheadand its state thehead state.
It is generally possible for two blocks to be valid and yet
reference the same prior block in what is known as afork.
This implies the possibility of two different heads, each
with their own state. While we know of no way to strictly
preclude this possibility, for the system to be useful we
must nonetheless attempt to minimize it. We therefore
strive to ensure that:
(1)It be generally unlikely for two heads to form.
(2)When two heads do form they be quickly resolved
into a single head.
(3)It be possible to identify a block not much older
than the head which we can be extremely confi-
dent will form part of the blockchain’s history in
perpetuity. When a block becomes identified as
such we call itfinalizedand this property natu-
rally extends to all of its ancestor blocks.
These goals are achieved through a combination of
two consensus mechanisms:Safrole, which governs the
(not-necessarily forkless) extension of the blockchain; and
Grandpa, which governs the finalization of some extension
into canonical history. Thus, the former delivers point
## 1,
the latter delivers point3and both are important for de-
livering point2. We describe these portions of the protocol
in detail in sections6and19respectively.
While Safrole limits forks to a large extent (through
cryptography, economics and common-time, below), there
may be times when we wish to intentionally fork since we
have come to know that a particular chain extension must
be reverted. In regular operation this should never hap-
pen, however we cannot discount the possibility of mali-
cious or malfunctioning nodes. We therefore define such
an extension as any which contains a block in which data
is reported whichany otherblock’s state has tagged as
invalid (see section
10on how this is done). We further
require that Grandpa not finalize any extension which con-
tains such a block. See section19for more information
here.
4.4.Time.We presume a pre-existing consensus over
time specifically for block production and import. While
this was not an assumption of Polkadot, pragmatic and
resilient solutions exist including thentpprotocol and
network. We utilize this assumption in only one way: we
require that blocks be considered temporarily invalid if
their timeslot is in the future. This is specified in detail
in section
## 6.
Formally, we define the time in terms of seconds passed
since the beginning of the
## J
amCommon Era, 1200utcon
## January 1, 2025.
## 8
Middayutcis selected to ensure that
all major timezones are on the same date at any exact
24-hour multiple from the beginning of the common era.
Formally, this value is denotedT.
4.5.Best block.Given the recognition of a number of
valid blocks, it is necessary to determine which should be
treated as the “best” block, by which we mean the most
recent block we believe will ultimately be within of all fu-
ture
## J
amchains. The simplest and least risky means of
doing this would be to inspect the Grandpa finality mech-
anism which is able to provide a block for which there is a
very high degree of confidence it will remain an ancestor
to any future chain head.
## 8
1,735,732,800 seconds after the Unix Epoch.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202510
However, in reducing the risk of the resulting block ul-
timately not being within the canonical chain, Grandpa
will typically return a block some small period older than
the most recently authored block. (Existing deployments
suggest around 1-2 blocks in the past under regular oper-
ation.) There are often circumstances when we may wish
to have less latency at the risk of the returned block not
ultimately forming a part of the future canonical chain.
E.g. we may be in a position of being able to author a
block, and we need to decide what its parent should be.
Alternatively, we may care to speculate about the most
recent state for the purpose of providing information to a
downstream application reliant on the state of
## J
am.
In these cases, we define the best block as the head of
the best chain, itself defined in section19.
4.6.Economics.The present work describes a crypto-
economic system, i.e. one combining elements of both
cryptography and economics and game theory to deliver
a self-sovereign digital service. In order to codify and ma-
nipulate economic incentives we define a token which is
native to the system, which we will simply calltokensin
the present work.
A value of tokens is generally referred to as abalance,
and such a value is said to be a member of the set of bal-
ances,N
## B
, which is exactly equivalent to the set of natu-
rals less than2
## 64
(i.e. 64-bit unsigned integers in coding
parlance). Formally:
## N
## B
## ≡N
## 2
## 64
## (4.21)
Though unimportant for the present work, we presume
that there be a standard named denomination for10
## 9
to-
kens. This is different to both Ethereum (which uses a
denomination of
## 10
## 18
), Polkadot (which uses a denomina-
tion of10
## 10
) and Polkadot’s experimental cousin Kusama
(which uses10
## 12
## ).
The fact that balances are constrained to being less
than2
## 64
implies that there may never be more than
around18×10
## 9
tokens (each divisible into portions of10
## −9
## )
within
## J
am. We would expect that the total number of
tokens ever issued will be a substantially smaller amount
than this.
We further presume that a number of constantprices
stated in terms of tokens are known. However we leave
the specific values to be determined in following work:
## B
## I
:the additional minimum balance implied for a
single item within a mapping.
## B
## L
:the additional minimum balance implied for a
single octet of data within a mapping.
## B
## S
:the minimum balance implied for a service.
4.7.The Virtual Machine and Gas.In the present
work, we presume the definition of aPolkadot Virtual
Machine(pvm). This virtual machine is based around
therisc-vinstruction set architecture, specifically the
rv64emvariant, and is the basis for introducing permis-
sionless logic into our state-transition function.
Thepvmis comparable to theevmdefined in the Yel-
low Paper, but somewhat simpler: the complex instruc-
tions for cryptographic operations are missing as are those
which deal with environmental interactions. Overall it is
far less opinionated since it alters a pre-existing general
purpose design,risc-v, and optimizes it for our needs.
This gives us excellent pre-existing tooling, sincepvmre-
mains essentially compatible withrisc-v, including sup-
port from the compiler toolkitllvmand languages such
as Rust and C++. Furthermore, the instruction set sim-
plicity whichrisc-vandpvmshare, together with the
register size (64-bit), active number (13) and endianness
(little) make it especially well-suited for creating eﬀicient
recompilers on to common hardware architectures.
Thepvmis fully defined in appendixA, but for contex-
tualization we will briefly summarize the basic invocation
functionΨwhich computes the resultant state of apvm
instance initialized with some registers (⟦N
## R
## ⟧
## 13
) andram
(M) and has executed for up to some amount of gas (N
## G
## ),
a number of approximately time-proportional computa-
tional steps:
## (4.22)
## Ψ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## B,N
## R
## ,N
## G
## ,
## ⟦N
## R
## ⟧
## 13
## ,M
## ⎫
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎭
## →
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ∎,☇,∞
## 
## ∪{
## F
## ,
## ̵
h}×N
## R
## ,
## N
## R
## ,Z
## G
## ,⟦N
## R
## ⟧
## 13
## ,M
## ⎫
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎭
We refer to the time-proportional computational steps
asgas(much like in theYP) and limit it to a 64-bit quan-
tity. We may use eitherN
## G
orZ
## G
to bound it, the first as
a prior argument since it is known to be positive, the latter
as a result where a negative value indicates an attempt to
execute beyond the gas limit. Within the context of the
pvm,ρ∈N
## G
is typically used to denote gas.
## (4.23)
## Z
## G
## ≡Z
## −2
## 63
## ...2
## 63
## ,N
## G
## ≡N
## 2
## 64
## ,N
## R
## ≡N
## 2
## 64
It is left as a rather important implementation detail to
ensure that the amount of time taken while computing the
functionΨ(...,ρ,...)has a maximum computation time
approximately proportional to the value ofρregardless of
other operands.
Thepvmis a very simpleriscregister machineand as
such has 13 registers, each of which is a 64-bit quantity,
denoted asN
## R
, a natural less than2
## 64
## .
## 9
Within the con-
text of thepvm,φ∈⟦N
## R
## ⟧
## 13
is typically used to denote the
registers.
## M≡
## ⎧
## ⎪
## ⎩
v∈B
## 2
## 32
,a∈⟦{W,R,∅}⟧
p
## ⎫
## ⎪
## ⎭
, p=
## 2
## 32
## Z
## P
## (4.24)
## Z
## P
## =2
## 12
## (4.25)
Thepvmassumes a simple pageableramof 32-bit ad-
dressable octets situated in pages ofZ
## P
## =4096octets
where each page may be either immutable, mutable or
inaccessible. TheramdefinitionMincludes two compo-
nents: a valuevand accessa. If the component is un-
specified while being subscripted then the value compo-
nent may be assumed. Within the context of the virtual
machine,μ∈Mis typically used to denoteram.
## V
μ
## ≡
## {
i
## S
μ
a
## [⌊
i
## ~
## Z
## P
## ⌋]
## ≠
## ∅}
## (4.26)
## V
## ∗
μ
≡{iSμ
a
## [⌊
i
## ~Z
## P
## ⌋]=W}(4.27)
We define two sets of indices for theramμ:V
μ
is the
set of indices which may be read from; andV
## ∗
μ
is the set
of indices which may be written to.
Invocation of thepvmhas an exit-reason as the first
item in the resultant tuple. It is either:
●Regular program termination caused by an ex-
plicit halt instruction,∎.
## 9
This is three fewer thanrisc-v’s 16, however the amount that program code output by compilers uses is 13 since two are reserved for
operating system use and the third is fixed as zero

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202511
●Irregular program termination caused by some ex-
ceptional circumstance,☇.
●Exhaustion of gas,∞.
●A page fault (attempt to access some address in
ramwhich is not accessible),
## F
. This includes the
address of the page at fault.
●An attempt at progressing a host-call,
## ̵
h
## . This
allows for the progression and integration of a
context-dependent state-machine beyond the reg-
ularpvm.
The full definition follows in appendixA.
4.8.Epochs and Slots.Unlike theYPEthereum with
its proof-of-work consensus system,
## J
amdefines a proof-of-
authority consensus mechanism, with the authorized val-
idators presumed to be identified by a set of public keys
and decided by astakingmechanism residing within some
system hosted by
## J
am. The staking system is out of scope
for the present work; instead there is anapiwhich may
be utilized to update these keys, and we presume that
whatever logic is needed for the staking system will be
introduced and utilize thisapias needed.
The Safrole mechanism subdivides time following gen-
esis into fixed lengthepochs with each epoch divided into
E=600timeslots each of uniform lengthP=6seconds,
given an epoch period ofE⋅P=3600seconds or one hour.
This six-second slot period represents the minimum
time between
## J
amblocks, and through Safrole we aim
to strictly minimize forks arising both due to contention
within a slot (where two valid blocks may be produced
within the same six-second period) and due to contention
over multiple slots (where two valid blocks are produced
in different time slots but with the same parent).
Formally when identifying a timeslot index, we use a
natural less than2
## 32
(in compute parlance, a 32-bit un-
signed integer) indicating the number of six-second times-
lots from the
## J
amCommon Era. For use in this context
we introduce the setN
## T
## :
## N
## T
## ≡N
## 2
## 32
## (4.28)
This implies that the lifespan of the proposed protocol
takes us to mid-August of the year 2840, which with the
current course that humanity is on should be ample.
4.9.The Core Model and Services.Whereas in the
Ethereum Yellow Paper when defining the state machine
which is held in consensus amongst all network partici-
pants, we presume that all machines maintaining the full
network state and contributing to its enlargement—or, at
least, hoping to—evaluate all computation. This “every-
body does everything” approach might be called theon-
chain consensus model. It is unfortunately not scalable,
since the network can only process as much logic in con-
sensus that it could hope any individual node is capable
of doing itself within any given period of time.
4.9.1.In-core Consensus.In the present work, we achieve
scalability of the work done through introducing a sec-
ond model for such computation which we call thein-core
consensus model. In this model, and under normal cir-
cumstances, only a subset of the network is responsible
for actually executing any given computation and assur-
ing the availability of any input data it relies upon to
others. By doing this and assuming a certain amount of
computational parallelism within the validator nodes of
the network, we are able to scale the amount of computa-
tion done in consensus commensurate with the size of the
network, and not with the computational power of any
single machine. In the present work we expect the net-
work to be able to do upwards of 300 times the amount
of computationin-coreas that which could be performed
by a single machine running the virtual machine at full
speed.
Since in-core consensus is not evaluated or verified by
all nodes on the network, we must find other ways to be-
come adequately confident that the results of the com-
putation are correct, and any data used in determining
this is available for a practical period of time. We do
this through a crypto-economic game of three stages called
guaranteeing,assuring,auditingand, potentially,judging.
Respectively, these attach a substantial economic cost to
the invalidity of some proposed computation; then a suﬀi-
cient degree of confidence that the inputs of the computa-
tion will be available for some period of time; and finally,
a suﬀicient degree of confidence that the validity of the
computation (and thus enforcement of the first guaran-
tee) will be checked by some party who we can expect to
be honest.
All execution done in-core must be reproducible by any
node synchronized to the portion of the chain which has
been finalized. Execution done in-core is therefore de-
signed to be as stateless as possible. The requirements for
doing it include only the refinement code of the service,
the code of the authorizer and any preimage lookups it
carried out during its execution.
When a work-report is presented on-chain, a specific
block known as thelookup-anchoris identified.  Cor-
rect behavior requires that this must be in the finalized
chain and reasonably recent, both properties which may
be proven and thus are acceptable for use within a con-
sensus protocol.
We describe this pipeline in detail in the relevant sec-
tions later.
4.9.2.On Services and Accounts.InYPEthereum, we
have two kinds of accounts:contract accounts(whose ac-
tions are defined deterministically based on the account’s
associated code and state) andsimple accountswhich act
as gateways for data to arrive into the world state and are
controlled by knowledge of some secret key. In
## J
am, all
accounts areservice accounts. Like Ethereum’s contract
accounts, they have an associated balance, some code and
state. Since they are not controlled by a secret key, they
do not need a nonce.
The question then arises: how can external data be fed
into the world state of
## J
am? And, by extension, how does
overall payment happen if not by deducting the account
balances of those who sign transactions? The answer to
the first lies in the fact that our service definition actu-
ally includesmultiplecode entry-points, one concerning
refinementand the other concerningaccumulation. The
former acts as a sort of high-performance stateless proces-
sor, able to accept arbitrary input data and distill it into
some much smaller amount of output data, which together
with some metadata is known as adigest. The latter code
is more stateful, providing access to certain on-chain func-
tionality including the possibility of transferring balance

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202512
and invoking the execution of code in other services. Be-
ing stateful this might be said to more closely correspond
to the code of an Ethereum contract account.
To understand how
## J
ambreaks up its service code is
to understand
## J
am’s fundamental proposition of general-
ity and scalability. All data extrinsic to
## J
amis fed into
the refinement code of some service. This code is not
executedon-chainbut rather is said to be executedin-
core. Thus, whereas the accumulator code is subject to
the same scalability constraints as Ethereum’s contract
accounts, refinement code is executed off-chain and sub-
ject to no such constraints, enabling
## J
amservices to scale
dramatically both in the size of their inputs and in the
complexity of their computation.
While refinement and accumulation take place in con-
sensus environments of a different nature, both are exe-
cuted by the members of the same validator set. The
## J
am
protocol through its rewards and penalties ensures that
code executedin-corehas a comparable level of crypto-
economic security to that executedon-chain, leaving the
primary difference between them one of scalability versus
synchroneity.
As for managing payment,
## J
amintroduces a new ab-
straction mechanism based around Polkadot’s Agile Core-
time. Within the Ethereum transactive model, the mecha-
nism of account authorization is somewhat combined with
the mechanism of purchasing blockspace, both relying on
a cryptographic signature to identify a single “transactor”
account. In
## J
am, these are separated and there is no such
concept of a “transactor”.
In place of Ethereum’s gas model for purchasing and
measuring blockspace,
## J
amhas the concept ofcoretime,
which is prepurchased and assigned to an authorization
agent. Coretime is analogous to gas insofar as it is the
underlying resource which is being consumed when utiliz-
ing
## J
am. Its procurement is out of scope in the present
work and is expected to be managed by a system parachain
operating within a parachains service itself blessed with a
number of cores for running such system services. The au-
thorization agent allows external actors to provide input
to a service without necessarily needing to identify them-
selves as with Ethereum’s transaction signatures. They
are discussed in detail in section
## 8.
5.The Header
We must first define the header in terms of its compo-
nents. The header comprises a parent hash and prior state
root (H
## P
andH
## R
), an extrinsic hashH
## X
, a time-slot in-
dexH
## T
, the epoch, winning-tickets and offenders markers
## H
## E
## ,H
## W
andH
## O
, a block author indexH
## I
and two Ban-
dersnatch signatures; the entropy-yieldingvrfsignature
## H
## V
and a block sealH
## S
. Headers may be serialized to an
octet sequence with and without the latter seal component
usingEandE
## U
respectively. Formally:
## (5.1)
## H≡(H
## P
## ,H
## R
## ,H
## X
## ,H
## T
## ,H
## E
## ,H
## W
## ,H
## O
## ,H
## I
## ,H
## V
## ,H
## S
## )
The blockchain is a sequence of blocks, each crypto-
graphically referencing some prior block by including a
hash derived from the parent’s header, all the way back to
some first block which references the genesis header. We
already presume consensus over this genesis headerH
## 0
and the state it represents defined asσ
## 0
## .
Excepting the Genesis header, all block headersHhave
an associated parent header, whose hash isH
## P
. We de-
note the parent headerH
## −
## =P(H):
## (5.2)H
## P
## ∈H,H
## P
## ≡H(E(P(H)))
Pis thus defined as being the mapping from one block
header to its parent block header. WithP, we are able to
define the set of ancestor headersA:
h∈A⇔h=H∨(∃i∈A∶h=P(i))(5.3)
We only require implementations to store headers of
ancestors which were authored in the previousL=24hours
of any blockBthey wish to validate.
The extrinsic hash is a Merkle commitment to the
block’s extrinsic data, taking care to allow for the possibil-
ity of reports to individually have their inclusion proven.
Given any blockB=(H,E), then formally:
## H
## X
## ∈H,H
## X
## ≡HEH
## #
## (a)(5.4)
wherea=[E
## T
## (E
## T
## ),E
## P
## (E
## P
),g,E
## A
## (E
## A
## ),E
## D
## (E
## D
## )](5.5)
andg=E(↕[(H(r),E
## 4
(t),↕a) S (r,t,a)<−E
## G
## ])(5.6)
A block may only be regarded as valid once the time-
slot indexH
## T
is in the past. It is always strictly greater
than that of its parent. Formally:
## (5.7)H
## T
## ∈N
## T
## ,  P(H)
## T
## <H
## T
## ∧H
## T
## ⋅P≤T
Blocks considered invalid by this rule may become valid
asTadvances.
The parent state rootH
## R
is the root of a Merkle trie
composed by the mapping of thepriorstate’s Merkle root,
which by definition is also the parent block’s posterior
state. This is a departure from both Polkadot and the Yel-
low Paper’s Ethereum, in both of which a block’s header
contains theposteriorstate’s Merkle root. We do this
to facilitate the pipelining of block computation and in
particular of Merklization.
## (5.8)
## H
## R
## ∈H,H
## R
## ≡M
σ
## (σ)
We assume the state-Merklization functionM
σ
is ca-
pable of transforming our stateσinto a 32-octet commit-
ment. See appendixDfor a full definition of these two
functions.
All blocks have an associated public key to identify the
author of the block. We identify this as an index into the
posterior current validator setκ
## ′
. We denote the Bander-
snatch key of the author asH
## A
though note that this is
merely an equivalence, and is not serialized as part of the
header.
## (5.9)
## H
## I
## ∈N
## V
## ,H
## A
## ≡κ
## ′
## [H
## I
## ]
b
5.1.The Markers.If not∅, then the epoch marker
specifies key and entropy relevant to the following epoch
in case the ticket contest does not complete adequately
(a very much unexpected eventuality).  Similarly, the
winning-tickets marker, if not∅, provides the series of
600 slot sealing “tickets” for the next epoch (see the next
section). Finally, the offenders marker is the sequence of
Ed25519 keys of newly misbehaving validators, to be fully
explained in section
## 10. Formally:
## (5.10)
## H
## E
## ∈
## ⎧
## ⎪
## ⎪
## ⎪
## ⎩
## H,H,D
## ⎧
## ⎪
## ⎪
## ⎩
## ∽
## H,
## ̄
## H
## ⎫
## ⎪
## ⎪
## ⎭
## I
## V
## ⎫
## ⎪
## ⎪
## ⎪
## ⎭
## ?,H
## W
## ∈⟦T⟧
## E
## ?,H
## O
## ∈
## C
## ̄
## H
## H
The terms are fully defined in sections6.6and10.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202513
6.Block Production and Chain Growth
As mentioned earlier,
## J
amis architected around a hy-
brid consensus mechanism, similar in nature to that of
Polkadot’sBabe/Grandpahybrid.
## J
am’s block produc-
tion mechanism, termed Safrole after the novel Sassafras
production mechanism of which it is a simplified variant, is
a stateful system rather more complex than the Nakamoto
consensus described in theYP.
The chief purpose of a block production consensus
mechanism is to limit the rate at which new blocks may be
authored and, ideally, preclude the possibility of “forks”:
multiple blocks with equal numbers of ancestors.
To achieve this, Safrole limits the possible author of
any block within any given six-second timeslot to a sin-
gle key-holder from within a prespecified set ofvalidators.
Furthermore, under normal operation, the identity of the
key-holder of any future timeslot will have a very high de-
gree of anonymity. As a side effect of its operation, we
can generate a high-quality pool of entropy which may be
used by other parts of the protocol and is accessible to
services running on it.
Because of its tightly scoped role, the core of Safrole’s
state,γ, is independent of the rest of the protocol. It in-
teracts with other portions of the protocol throughιand
κ, the prospective and active sets of validator keys re-
spectively;τ, the most recent block’s timeslot; andη, the
entropy accumulator.
The Safrole protocol generates, once per epoch, a se-
quence ofEsealing keys, one for each potential block
within a whole epoch. Each block header includes its
timeslot indexH
## T
(the number of six-second periods since
the
## J
amCommon Era began) and a valid seal signature
## H
## S
, signed by the sealing key corresponding to the times-
lot within the aforementioned sequence. Each sealing key
is in fact a pseudonym for some validator which was agreed
the privilege of authoring a block in the corresponding
timeslot.
In order to generate this sequence of sealing keys in
regular operation, and in particular to do so without mak-
ing public the correspondence relation between them and
the validator set, we use a novel cryptographic structure
known as a Ringvrf, utilizing the Bandersnatch curve.
Bandersnatch Ringvrfallows for a proof to be provided
which simultaneously guarantees the author controlled a
key within a set (in our case validators), and secondly pro-
vides an output, an unbiasable deterministic hash giving
us a secure verifiable random function (vrf). This anony-
mous and secure random output is aticketand validators’
tickets with the best score define the new sealing keys al-
lowing the chosen validators to exercise their privilege and
create a new block at the appropriate time.
6.1.Timekeeping.Here,τdefines the most recent
block’s slot index, which we transition to the slot index
as defined in the block’s header:
(6.1)τ∈N
## T
,  τ
## ′
## ≡H
## T
We track the slot index in state asτin order that we
are able to easily both identify a new epoch and deter-
mine the slot at which the prior block was authored. We
denoteeas the prior’s epoch index andmas the prior’s
slot phase index within that epoch ande
## ′
andm
## ′
are the
corresponding values for the present block:
leteRm=
τ
## E
,  e
## ′
## Rm
## ′
## =
τ
## ′
## E
## (6.2)
6.2.Safrole Basic State.We restateγinto a number
of components:
γ≡
## ⎧
## ⎩
γ
## P
, γ
## Z
, γ
## S
, γ
## A
## ⎫
## ⎭
## (6.3)
γ
## Z
is the epoch’s root, a Bandersnatch ring root com-
posed with the one Bandersnatch key of each of the next
epoch’s validators, defined inγ
## P
(itself defined in the next
section).
γ
## Z
## ∈
## ○
## B(6.4)
## Finally,γ
## A
is the ticket accumulator, a series of highest-
scoring ticket identifiers to be used for the next epoch.γ
## S
is the current epoch’s slot-sealer series, which is either a
full complement ofEtickets or, in the case of a fallback
mode, a series ofEBandersnatch keys:
γ
## A
## ∈⟦T⟧
## ∶E
,  γ
## S
## ∈⟦T⟧
## E
## ∪D
## ∽
## HI
## E
## (6.5)
Here,Tis used to denote the set oftickets, a combi-
nation of a verifiably random ticket identifieryand the
ticket’s entry-indexe:
## T≡
## ⎧
## ⎩
y∈H, e∈N
## N
## ⎫
## ⎭
## (6.6)
As we state in section6.4, Safrole requires that every
block headerHcontain a valid sealH
## S
, which is a Ban-
dersnatch signature for a public key at the appropriate
indexmof the current epoch’s seal-key series, present in
state asγ
## S
## .
6.3.Key Rotation.In addition to the active set of val-
idator keysκand staging setι, internal to the Safrole
state we retain a pending setγ
## P
. The active set is the
set of keys identifying the nodes which are currently priv-
ileged to author blocks and carry out the validation pro-
cesses, whereas the pending setγ
## P
, which is reset toι
at the beginning of each epoch, is the set of keys which
will be active in the next epoch and which determine the
Bandersnatch ring root which authorizes tickets into the
sealing-key contest for the next epoch.
ι
## ∈
## ⟦
## K
## ⟧
## V
,  γ
## P
## ∈⟦
## K
## ⟧
## V
,  κ
## ∈
## ⟦
## K
## ⟧
## V
,  λ
## ∈
## ⟦
## K
## ⟧
## V
## (6.7)
We must introduceK, the set of validator key tuples.
This is a combination of a set of cryptographic public keys
and metadata which is an opaque octet sequence, but uti-
lized to specify practical identifiers for the validator, not
least a hardware address.
The set of validator keys itself is equivalent to the set of
336-octet sequences. However, for clarity, we divide the
sequence into four easily denoted components. For any
validator keyk, the Bandersnatch key is denotedk
b
, and
is equivalent to the first 32-octets; the Ed25519 key,k
e
, is
the second 32 octets; theblskey denotedk
l
is equivalent
to the following 144 octets, and finally the metadatak
m
is the last 128 octets. Formally:
## K≡B
## 336
## (6.8)
∀k∈K∶k
b
## ∈
## ∽
## H≡k
## 0⋅⋅⋅+32
## (6.9)
∀k∈K∶k
e
## ∈
## ̄
## H≡k
## 32⋅⋅⋅+32
## (6.10)
∀k∈K∶k
l
## ∈
## BLS
## B≡k
## 64⋅⋅⋅+144
## (6.11)
∀k∈K∶k
m
## ∈B
## 128
## ≡k
## 208⋅⋅⋅+128
## (6.12)

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202514
With a new epoch under regular conditions, validator
keys get rotated and the epoch’s Bandersnatch key root is
updated intoγ
## ′
## Z
## :
## 
γ
## ′
## P
## ,κ
## ′
## ,λ
## ′
## ,γ
## ′
## Z
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
(Φ(ι),γ
## P
## ,κ,z)ife
## ′
## >e
## (γ
## P
## ,κ,λ,γ
## Z
## )otherwise
## (6.13)
wherez=O
## 
k
b
## T
k<−γ
## ′
## P
## 
## Φ(k)≡
## [0,0,...]ifk
e
## ∈ψ
## ′
## O
kotherwise
## ¡ Wk<−k	(6.14)
Note that on epoch changes the posterior queued val-
idator key setγ
## ′
## P
is defined such that incoming keys be-
longing to the offendersψ
## ′
## O
are replaced with a null key
containing only zeroes. The origin of the offenders is ex-
plained in section10.
6.4.Sealing and Entropy Accumulation.The header
must contain a valid seal and validvrfoutput. These are
two signatures both using the current slot’s seal key; the
message data of the former is the header’s serialization
omitting the seal componentH
## S
, whereas the latter is
used as a bias-resistant entropy source and thus its mes-
sage must already have been fixed: we use the entropy
stemming from thevrfof the seal signature. Formally:
leti=γ
## ′
## S
## [H
## T
## ]
## ↺
## ∶
γ
## ′
## S
## ∈⟦T⟧Ô⇒
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
i
y
## =Y
## (
## H
## S
## )
## ,
## H
## S
## ∈
## ∽
## V
## E
## U
## (H)
## H
## A
a
## X
## T
## ⌢η
## ′
## 3
i
e
f
## ,
## T=1
## (6.15)
γ
## ′
## S
## ∈D
## ∽
## HIÔ⇒
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
i=H
## A
## ,
## H
## S
## ∈
## ∽
## V
## E
## U
## (H)
## H
## A
a
## X
## F
## ⌢η
## ′
## 3
f
## ,
## T=0
## (6.16)
## H
## V
## ∈
## ∽
## V
## []
## H
## A
## ⟨X
## E
## ⌢Y(H
## S
## )⟩(6.17)
## X
## E
## =$jam_entropy(6.18)
## X
## F
## =$jam_fallback_seal(6.19)
## X
## T
## =$jam_ticket_seal(6.20)
Sealing using the ticket is of greater security, and we
utilize this knowledge when determining a candidate block
on which to extend the chain, detailed in section
## 19. We
thus note that the block was sealed under the regular se-
curity with the boolean markerT. We define this only for
the purpose of ease of later specification.
In addition to the entropy accumulatorη
## 0
, we retain
three additional historical values of the accumulator at
the point of each of the three most recently ended epochs,
η
## 1
## ,η
## 2
andη
## 3
. The second-oldest of theseη
## 2
is utilized to
help ensure future entropy is unbiased (see equation
## 6.29)
and seed the fallback seal-key generation function with
randomness (see equation6.24). The oldest is used to re-
generate this randomness when verifying the seal above
(see equations
## 6.16and6.15).
η∈⟦H⟧
## 4
## (6.21)
η
## 0
defines the state of the randomness accumulator to
which the provably random output of thevrf, the signa-
ture over some unbiasable input, is combined each block.
η
## 1
## ,η
## 2
andη
## 3
meanwhile retain the state of this accumu-
lator at the end of the three most recently ended epochs
in order.
η
## ′
## 0
≡H(η
## 0
## ⌢Y(H
## V
## ))
## (6.22)
On an epoch transition (identified as the condition
e
## ′
>e), we therefore rotate the accumulator value into
the historyη
## 1
## ,η
## 2
andη
## 3
## :
## 
η
## ′
## 1
## ,η
## ′
## 2
## ,η
## ′
## 3
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## (η
## 0
## ,η
## 1
## ,η
## 2
## )ife
## ′
## >e
## (η
## 1
## ,η
## 2
## ,η
## 3
## )otherwise
## (6.23)
6.5.The Slot Key Sequence.The posterior slot key
sequenceTis one of three expressions depending on the
circumstance of the block. If the block is not the first in
an epoch, then it remains unchanged from the priorγ
## S
## .
If the block signals the next epoch (by epoch index) and
the previous block’s slot was within the closing period of
the previous epoch, then it takes the value of the prior
ticket accumulatorγ
## A
. Otherwise, it takes the value of
the fallback key sequence. Formally:
γ
## ′
## S
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## Z(γ
## A
## )ife
## ′
=e+1∧m≥Y∧Sγ
## A
## S=E
γ
## S
ife
## ′
## =e
## F(η
## ′
## 2
## ,κ
## ′
## )otherwise
## (6.24)
Here, we useZas the outside-in sequencer function,
defined as follows:
## (6.25)
## Z∶
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## ⟦T⟧
## E
## →⟦T⟧
## E
s↦
## 
s
## 0
## ,s
SsS−1
## ,s
## 1
## ,s
SsS−2
## ,...
## 
Finally,Fis the fallback key sequence function which
selects an epoch’s worth of validator Bandersnatch keys
## (D
## ∽
## HI
## E
) from the validator key setkusing the entropy col-
lected on-chainr:
## (6.26)F∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## H,⟦K⟧
## ⎫
## ⎭
## →D
## ∽
## HI
## E
## (r,k)↦k
## E
## −1
## 4
## 
H(r⌢E
## 4
## (i))
## ...4
## 
## ↺
b
Ui∈N
## E
## 
6.6.The Markers.The epoch and winning-tickets
markers are information placed in the header in order to
minimize data transfer necessary to determine the valida-
tor keys associated with any given epoch. They are partic-
ularly useful to nodes which do not synchronize the entire
state for any given block since they facilitate the secure
tracking of changes to the validator key sets using only
the chain of headers.
As mentioned earlier, the header’s epoch markerH
## E
is
either empty or, if the block is the first in a new epoch,
then a tuple of the next and current epoch randomness,
along with a sequence of tuples containing both Bander-
snatch keys and Ed25519 keys for each validator defining
the validator keys beginning in the next epoch. Formally:
## H
## E
## ≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## (η
## 0
## ,η
## 1
## ,[(k
b
## ,k
e
## ) Sk<
## −γ
## ′
## P
## ])ife
## ′
## >e
## ∅otherwise
## (6.27)
The winning-tickets markerH
## W
is either empty or, if
the block is the first after the end of the submission period
for tickets and if the ticket accumulator is saturated, then
the final sequence of ticket identifiers. Formally:
## H
## W
## ≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## Z(γ
## A
## )ife
## ′
=e∧m<Y≤m
## ′
∧Sγ
## A
## S=E
## ∅otherwise
## (6.28)

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202515
6.7.The Extrinsic and Tickets.The extrinsicE
## T
is a
sequence of proofs of valid tickets; a ticket implies an entry
in our epochal “contest” to determine which validators are
privileged to author a block for each timeslot in the follow-
ing epoch. Tickets specify an entry index together with a
proof of ticket’s validity. The proof implies a ticket iden-
tifier, a high-entropy unbiasable 32-octet sequence, which
is used both as a score in the aforementioned contest and
as input to the on-chainvrf.
Towards the end of the epoch (i.e.Yslots from the
start) this contest is closed implying successive blocks
within the same epoch must have an empty tickets extrin-
sic. At this point, the following epoch’s seal key sequence
becomes fixed.
We define the extrinsic as a sequence of proofs of valid
tickets, each of which is a tuple of an entry index (a nat-
ural number less thanN) and a proof of ticket validity.
## Formally:
## E
## T
## ∈E
## ⎧
## ⎪
## ⎪
## ⎪
## ⎩
e∈N
## N
, p∈
## ○
## V
## []
γ
## ′
## Z
a
## X
## T
## ⌢η
## ′
## 2
e
f
## ⎫
## ⎪
## ⎪
## ⎪
## ⎭
## J(6.29)
## SE
## T
## S≤
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## Kifm
## ′
## <Y
## 0otherwise
## (6.30)
We definenas the set of new tickets, with the ticket
identifier, a hash, defined as the output component of the
## Bandersnatch Ringvrfproof:
n≡[(y
## ▸
## ▸
## Y(i
p
), e
## ▸
## ▸
i
e
) Si<−E
## T
## ](6.31)
The tickets submitted via the extrinsic must already
have been placed in order of their implied identifier. Du-
plicate identifiers are never allowed lest a validator submit
the same ticket multiple times:
n=[x∈n
## _
## _
x
y
## ](6.32)
## {x
y
## Sx∈n}⫰{x
y
## Sx∈γ
## A
## }(6.33)
The new ticket accumulatorγ
## ′
## A
is constructed by merg-
ing new tickets into the previous accumulator value (or the
empty sequence if it is a new epoch):
## (6.34)
γ
## ′
## A
## ≡
## ÐÐÐÐÐÐÐÐÐÐÐÐÐÐÐÐÐÐÐÐ→
## ⎡
## ⎢
## ⎢
## ⎢
## ⎢
## ⎣
x∈n∪
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## ∅ife
## ′
## >e
γ
## A
otherwise
## ^
## ^
## ^
## ^
## ^
## ^
x
y
## ⎤
## ⎥
## ⎥
## ⎥
## ⎥
## ⎦
## E
The maximum size of the ticket accumulator isE. On
each block, the accumulator becomes the lowest items of
the sorted union of tickets from prior accumulatorγ
## A
and
the submitted tickets. It is invalid to include useless tick-
ets in the extrinsic, so all submitted tickets must exist in
their posterior ticket accumulator. Formally:
n⊆γ
## ′
## A
## (6.35)
Note that it can be shown that in the case of an empty
extrinsicE
## T
=[], as implied bym
## ′
≥Y, and unchanged
epoch (e
## ′
=e), thenγ
## ′
## A
## =γ
## A
## .
7.Recent History
We retain in state information on the most recentH
blocks. This is used to preclude the possibility of dupli-
cate or out of date work-reports from being submitted.
β≡(β
## H
## ,β
## B
## )
## (7.1)
β
## H
## ∈⟦
## ⎧
## ⎩
h∈H,s∈H,b∈H,p∈jH→Ho
## ⎫
## ⎭
## ⟧
## ∶H
## (7.2)
β
## B
## ∈⟦H?⟧(7.3)
θ∈⟦(N
## S
## ,H)⟧(7.4)
For each recent block, we retain its header hash, its
state root, its accumulation-resultmmband the cor-
responding work-package hashes of each item reported
(which is no more than the total number of cores,C=341).
During the accumulation stage, a value with the par-
tial transition of this state is provided which contains the
correction for the newly-known state-root of the parent
block:
## (7.5)β
## †
## H
## ≡β
## H
exceptβ
## †
## H
[Sβ
## H
## S−1]
s
## =H
## R
We define the new Accumulation Output Logβ
## B
## . This
is formed from the block’s accumulation-output sequence
θ
## ′
(defined in section12), taking its root using the basic bi-
nary Merklization function (M
## B
, defined in appendixE)
and appending it to the previous log value with themmb
append function (defined in appendixE.2). Throughout,
the Keccak hash function is used to maximize compatibil-
ity with legacy systems:
lets=
## 
## E
## 4
(s)⌢E(h)
## T
## (s,h)<−θ
## ′
## 
## (7.6)
β
## ′
## B
≡A(β
## B
## ,M
## B
(s,H
## K
## ),H
## K
## )(7.7)
The final state transition forβ
## H
appends a new item
including the new block’s header hash, a Merkle commit-
ment to the block’s Accumulation Output Log and the set
of work-reports made into it (for which we use the guar-
antees extrinsic,E
## G
## ). Formally:
## (7.8)
β
## ′
## H
## ≡
## ←ÐÐÐÐÐÐÐÐÐÐÐÐÐÐÐÐÐÐÐÐÐÐÐÐÐ
β
## †
## H
## 
p,h
## ▸
## ▸
H(H),s
## ▸
## ▸
## H
## 0
## ,b
## ▸
## ▸
## M
## R
## 
β
## ′
## B
## 
## H
wherep={ (((g
r
## )
s
## )
p
## ↦((g
r
## )
s
## )
e
) Sg∈E
## G
## }
The new state-trie root is the zero hash,H
## 0
, which is
inaccurate but safe sinceβ
## ′
is not utilized except to define
the next block’sβ
## †
, which contains a corrected value for
this, as per equation
## 7.5.
8.Authorization
We have previously discussed the model of work-
packages and services in section
4.9, however we have yet
to make a substantial discussion of exactly how somecore-
timeresource may be apportioned to some work-package
and its associated service. In theYPEthereum model, the
underlying resource, gas, is procured at the point of intro-
duction on-chain and the purchaser is always the same
agent who authors the data which describes the work to
be done (i.e. the transaction). Conversely, in Polkadot the
underlying resource, a parachain slot, is procured with a
substantial deposit for typically 24 months at a time and
the procurer, generally a parachain team, will often have
no direct relation to the author of the work to be done
(i.e. a parachain block).
On a principle of flexibility, we would wish
## J
amca-
pable of supporting a range of interaction patterns both
Ethereum-style and Polkadot-style. In an effort to do so,
we introduce theauthorization system, a means of disen-
tangling the intention of usage for some coretime from the
specification and submission of a particular workload to
be executed on it. We are thus able to disassociate the
purchase and assignment of coretime from the specific de-
termination of work to be done with it, and so are able to
support both Ethereum-style and Polkadot-style interac-
tion patterns.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202516
8.1.Authorizers and Authorizations.The authoriza-
tion system involves three key concepts:Authorizers,To-
kensandTraces. A Token is simply a piece of opaque
data to be included with a work-package to help make
an argument that the work-package should be authorized.
Similarly, a Trace is a piece of opaque data which helps
characterize or describe some successful authorization. An
Authorizer meanwhile, is a piece of logic which executes
within some pre-specified and well-known computational
limits and determines whether a work-package—including
its Token—is authorized for execution on some particular
core and yields a Trace on success.
Authorizers are identified as the hash of theirpvmcode
concatenated with their Configuration blob, the latter be-
ing, like Tokens and Traces, opaque data meaningful to the
pvmcode. The process by which work-packages are de-
termined to be authorized (or not) is not the competence
of on-chain logic and happens entirely in-core and as such
is discussed in section
14.3. However, on-chain logic must
identify each set of authorizers assigned to each core in
order to verify that a work-package is legitimately able
to utilize that resource. It is this subsystem we will now
define.
8.2.Pool and Queue.We define the set of authorizers
allowable for a particular corecas theauthorizer pool
α[c]. To maintain this value, a further portion of state is
tracked for each core: the core’s currentauthorizer queue
φ[c], from which we draw values to fill the pool. Formally:
## (8.1)α∈
## C
## ⟦H⟧
## ∶O
## H
## C
,   φ∈
## C
## ⟦H⟧
## Q
## H
## C
Note: The portion of stateφmay be altered only
through an exogenous call made from the accumulate logic
of an appropriately privileged service.
The state transition of a block involves placing a new
authorization into the pool from the queue:
∀c∈N
## C
## ∶α
## ′
## [c]≡
## ←ÐÐÐÐÐÐÐÐÐÐÐÐÐ
## F(c)
φ
## ′
[c][H
## T
## ]
## ↺
## O
## (8.2)
## F(c)≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
α[c]m{ (g
r
## )
a
}if∃g∈E
## G
## ∶(g
r
## )
c
## =c
α[c]otherwise
## (8.3)
## Sinceα
## ′
is dependent onφ
## ′
, practically speaking, this
step must be computed after accumulation, the stage in
whichφ
## ′
is defined. Note that we utilize the guarantees ex-
trinsicE
## G
to remove the oldest authorizer which has been
used to justify a guaranteed work-package in the current
block. This is further defined in equation
## 11.23.
9.Service Accounts
As we already noted, a service in
## J
amis somewhat
analogous to a smart contract in Ethereum in that it in-
cludes amongst other items, a code component, a storage
component and a balance. Unlike Ethereum, the code is
split over two isolated entry-points each with their own
environmental conditions; one,Refinement, is essentially
stateless and happens in-core, and the other,Accumula-
tion, which is stateful and happens on-chain. It is the
latter which we will concern ourselves with now.
Service accounts are held in state underδ, a partial
mapping from a service identifierN
## S
into a tuple of named
elements which specify the attributes of the service rele-
vant to the
## J
amprotocol. Formally:
## N
## S
## ≡N
## 2
## 32
## (9.1)
δ∈jN
## S
→Ao(9.2)
The service account is defined as the tuple of storage
dictionarys, preimage lookup dictionariespandl, code
hashc, balanceband gratis storage offsetf, as well as the
two code gas limitsg&m. We also record certain usage
characteristics concerning the account: the time slot at
creationr, the time slot at the most recent accumulation
aand the parent servicep. Formally:
## A≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
s∈jB→Bo,p∈jH→Bo,
l∈
k
## ⎧
## ⎩
## H,N
## L
## ⎫
## ⎭
## →⟦N
## T
## ⟧
## ∶3
p
## ,
f∈N
## B
, c∈H, b∈N
## B
, g∈N
## G
## ,
m∈N
## G
, r∈N
## T
, a∈N
## T
, p∈N
## S
## ⎫
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎭
## (9.3)
Thus, the balance of the service of indexswould be
denotedδ[s]
b
and the storage item of keyk∈Bfor that
service is writtenδ[s]
s
## [k].
9.1.Code and Gas.The code and associated metadata
of a service account is identified by a hash which, if the ser-
vice is to be functional, must be present within its preim-
age lookup (see section9.2) and have a preimage which is
a proper encoding of the two blobs. We thus define the
actual codecand metadatam:
∀a∈A∶(a
m
## ,a
c
## )≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
(m,c)ifE(↕m,c)=a
p
## [a
c
## ]
## (∅,∅)otherwise
## (9.4)
There are two entry-points in the code:
0refine:Refinement, executed in-core and state-
less.
## 10
1accumulate:Accumulation,  executed on-chain
and stateful.
Refinement and accumulation are described in more
detail in sections14.4and12.2respectively.
As stated in appendixA, execution time in the
## J
am
virtual machine is measured deterministically in units of
gas, represented as a natural number less than2
## 64
and
formally denotedN
## G
. We may also useZ
## G
to denote the
setZ
## −2
## 63
## ...2
## 63
if the quantity may be negative. There are
two limits specified in the account, which determine the
minimum gas required in order to execute theAccumu-
lateentry-point of the service’s code.gis the minimum
gas required per work-item, whilemis the minimum gas
required per deferred-transfer.
9.2.Preimage Lookups.In addition to storing data in
arbitrary key/value pairs available only on-chain, an ac-
count may also solicit data to be made available also in-
core, and thus available to the Refine logic of the service’s
code. State concerning this facility is held under the ser-
vice’spandlcomponents.
There are several differences between preimage-lookups
and storage.  Firstly, preimage-lookups act as a map-
ping from a hash to its preimage, whereas general storage
maps arbitrary keys to values. Secondly, preimage data
is supplied extrinsically, whereas storage data originates
as part of the service’s accumulation. Thirdly preimage
data, once supplied, may not be removed freely; instead
## 10
Technically there is some small assumption of state, namely that some modestly recent instance of each service’s preimages. The
specifics of this are discussed in section14.3.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202517
it goes through a process of being marked as unavailable,
and only after a period of time may it be removed from
state. This ensures that historical information on its exis-
tence is retained. The final point especially is important
since preimage data is designed to be queried in-core, un-
der the Refine logic of the service’s code, and thus it is
important that the historical availability of the preimage
is known.
We begin by reformulating the portion of state concern-
ing our data-lookup system. The purpose of this system
is to provide a means of storing static data on-chain such
that it may later be made available within the execution
of any service code as a function accepting only the hash
of the data and its length in octets.
During the on-chain execution of theAccumulatefunc-
tion, this is trivial to achieve since there is inherently a
state which all validators verifying the block necessarily
have complete knowledge of, i.e.σ. However, for the in-
core execution ofRefine, there is no such state inherently
available to all validators; we thus name a historical state,
thelookup anchorwhich must be considered recently final-
ized before the work’s implications may be accumulated
hence providing this guarantee.
By retaining historical information on its availability,
we become confident that any validator with a recently fi-
nalized view of the chain is able to determine whether any
given preimage was available at any time within the period
where auditing may occur. This ensures confidence that
judgments will be deterministic even without consensus
on chain state.
Restated, we must be able to define somehistorical
lookup functionΛwhich determines whether the preim-
age of some hash was available for lookup by some service
account at some timeslot, and if so, provide it:
## (9.5)
## Λ∶
## ⎧
## ⎩
## A,N
## (H
## T
## −D)...H
## T
## ,H
## ⎫
## ⎭
## →B?
(a,t,H(p))↦v∶v∈{p,∅}
This function is defined shortly below in equation
## 9.7.
The preimage lookup for some service of indexsis de-
notedδ[s]
p
is a dictionary mapping a hash to its corre-
sponding preimage. Additionally, there is metadata asso-
ciated with the lookup denotedδ[s]
l
which is a dictionary
mapping some hash and presupposed length into historical
information.
9.2.1.Invariants.The state of the lookup system natu-
rally satisfies a number of invariants. Firstly, any preim-
age value must correspond to its hash, equation
## 9.6. Sec-
ondly, a preimage value being in state implies that its
hash and length pair has some associated status, also in
equation
## 9.6. Formally:
(9.6)∀a∈A,(h↦d)∈a
p
⇒h=H(d)∧(h,SdS)∈K(a
l
## )
## 9.2.2.
## Semantics.
The historical status component
h
## ∈
## ⟦N
## T
## ⟧
## ∶3
is a sequence of up to three time slots and the
cardinality of this sequence implies one of four modes:
●h=⟦⟧: The preimage isrequested, but has not yet
been supplied.
●h∈⟦N
## T
## ⟧
## 1
: The preimage isavailableand has been
from timeh
## 0
## .
●h∈⟦N
## T
## ⟧
## 2
: The previously available preimage is
nowunavailablesince timeh
## 1
. It had been avail-
able from timeh
## 0
## .
●h∈⟦N
## T
## ⟧
## 3
: The preimage isavailableand has been
from timeh
## 2
. It had previously been available
from timeh
## 0
until timeh
## 1
## .
The historical lookup functionΛmay now be defined
as:
## (9.7)
## Λ∶
## ⎧
## ⎩
## A,N
## T
## ,H
## ⎫
## ⎭
## →B?
## Λ(a,t,h)≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
a
p
[h]ifh∈K(a
p
)∧I(a
l
[h,Sa
p
[h]S],t)
## ∅otherwise
whereI(l,t)=
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## if[]=l
x≤tif[x]=l
x≤t<yif[x,y]=l
x≤t<y∨z≤tif[x,y,z]=l
9.3.Account Footprint and Threshold Balance.We
define the dependent valuesiandoas the storage foot-
print of the service, specifically the number of items in
storage and the total number of octets used in storage.
They are defined purely in terms of the storage map of a
service, and it must be assumed that whenever a service’s
storage is changed, these change also.
Furthermore, as we will see in the account serialization
function in sectionC, these are expected to be found ex-
plicitly within the Merklized state data. Because of this
we make explicit their set.
We may then define a third dependent termt, the min-
imum, orthreshold, balance needed for any given service
account in terms of its storage footprint.
∀a∈V(δ)∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
a
i
## ∈N
## 2
## 32
≡2⋅Sa
l
S+Sa
s
## S
a
o
## ∈N
## 2
## 64
## ≡
## ∑
(h,z)∈K(a
l
## )
## 81+z
## +
## ∑
## (x,y)∈a
s
34+SyS+SxS
a
t
## ∈N
## B
≡max(0,B
## S
## +B
## I
## ⋅a
i
## +B
## L
## ⋅a
o
## −a
f
## )
## (9.8)
9.4.Service Privileges.
## J
amincludes the ability to be-
stow privileges on a number of services. The portion of
state in which this is held is denotedχand includes five
kinds of privilege. The first,χ
## M
, is the index of theman-
agerservice which is the service able to effect an alteration
ofχfrom block to block as well as bestow services with
storage deposit credits. The next,χ
## V
, is able to setι.
## Thenχ
## R
alone is able to create new service accounts with
indices in the protected range. The following,χ
## A
, are the
service indices capable of altering the authorizer queueφ,
one for each core.
## Finally,χ
## Z
is a small dictionary containing the indices
of services which automatically accumulate in each block
together with a basic amount of gas with which each ac-
cumulates. Formally:
χ≡
## ⎧
## ⎩
χ
## M
## ,χ
## V
## ,χ
## R
## ,χ
## A
## ,χ
## Z
## ⎫
## ⎭
## (9.9)
χ
## M
## ∈N
## S
,   χ
## V
## ∈N
## S
,   χ
## R
## ∈N
## S
## (9.10)
χ
## A
## ∈⟦N
## S
## ⟧
## C
,   χ
## Z
∈jN
## S
## →N
## G
o(9.11)
10.Disputes, Verdicts and Judgments
## J
amprovides a means of recordingjudgments: conse-
quential votes amongst most of the validators over the
validity of awork-report(a unit of work done within
## J
am,
see section
11). Such collections of judgments are known

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202518
asverdicts.
## J
amalso provides a means of registeringof-
fenses, judgments and guarantees which dissent with an
establishedverdict. Together these form thedisputessys-
tem.
The registration of a verdict is not expected to happen
very often in practice, however it is an important security
backstop for removing and banning invalid work-reports
from the processing pipeline as well as removing trouble-
some keys from the validator set where there is consen-
sus over their malfunction. It also helps coordinate nodes
to revert chain-extensions containing invalid work-reports
and provides a convenient means of aggregating all offend-
ing validators for punishment in a higher-level system.
Judgement statements come about naturally as part
of the auditing process and are expected to be positive,
further aﬀirming the guarantors’ assertion that the work-
report is valid. In the event of a negative judgment, then
all validators audit said work-report and we assume a ver-
dict will be reached. Auditing and guaranteeing are off-
chain processes properly described in sections
## 14and17.
A judgment against a report implies that the chain is
already reverted to some point prior to the accumulation
of said report, usually forking at the block immediately
prior to that at which accumulation happened. The spe-
cific strategy for chain selection is described fully in section
- Authoring a block with a non-positive verdict has the
effect of cancelling its imminent accumulation, as can be
seen in equation
## 10.15.
Registering a verdict also has the effect of placing a
permanent record of the event on-chain and allowing any
offending keys to be placed on-chain both immediately or
in forthcoming blocks, again for permanent record.
Having a persistent on-chain record of misbehavior is
helpful in a number of ways. It provides a very simple
means of recognizing the circumstances under which ac-
tion against a validator must be taken by any higher-level
validator-selection logic. Should
## J
ambe used for a public
network such asPolkadot, this would imply the slashing of
the offending validator’s stake on the staking parachain.
As mentioned, recording reports found to have a high
confidence of invalidity is important to ensure that said
reports are not allowed to be resubmitted. Conversely,
recording reports found to be valid ensures that additional
disputes cannot be raised in the future of the chain.
10.1.The State.Thedisputesstate includes four items,
three of which concern verdicts: a good-set (ψ
## G
), a bad-
set (ψ
## B
) and a wonky-set (ψ
## W
) containing the hashes
of all work-reports which were respectively judged to be
correct, incorrect or that it appears impossible to judge.
The fourth item, the punish-set (ψ
## O
), is a set of Ed25519
keys representing validators which were found to have mis-
judged a work-report.
## (10.1)
ψ≡(ψ
## G
## ,ψ
## B
## ,ψ
## W
## ,ψ
## O
## )
10.2.Extrinsic.The disputes extrinsicE
## D
is functional
grouping of three otherwise independent extrinsics. It
comprisesverdictsE
## V
,culpritsE
## C
, andfaultsE
## F
## . Ver-
dicts are a compilation of judgments coming from exactly
two-thirds plus one of either the active validator set or the
previous epoch’s validator set, i.e. the Ed25519 keys ofκ
orλ. Culprits and faults are proofs of the misbehavior
of one or more validators, respectively either by guaran-
teeing a work-report found to be invalid, or by signing
a judgment found to be contradiction to a work-report’s
validity. Both of these are considered a kind ofoffense.
## Formally:
## (10.2)
## E
## D
## ≡
## ⎧
## ⎩
## E
## V
## ,E
## C
## ,E
## F
## ⎫
## ⎭
whereE
## V
## ∈E
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## H,
τ
## E
## −N
## 2
## ,
## C
## ⎧
## ⎩
## {⊺,},N
## V
## ,
## ̄
## V
## ⎫
## ⎭
## H
## ⌊
## 2
## ~3V⌋+1
## ⎫
## ⎪
## ⎪
## ⎪
## ⎪
## ⎭
## J
andE
## C
## ∈
## C
## ⎧
## ⎩
## H,
## ̄
## H,
## ̄
## V
## ⎫
## ⎭
## H
## ,E
## F
## ∈
## C
## ⎧
## ⎩
## H,{⊺,},
## ̄
## H,
## ̄
## V
## ⎫
## ⎭
## H
The signatures of all judgments must be valid in terms
of one of the two allowed validator key-sets, identified by
the verdict’s second term which must be either the epoch
index of the prior state or one less. Formally:
∀(r,a,j)∈E
## V
## ,∀(v,i,s)∈j∶s∈
## ̄
## V
k[i]
e
## ⟨X
v
## ⌢r⟩
wherek=
## ⎧
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎩
κifa=
τ
## E
## 
λotherwise
## (10.3)
## X
## ⊺
≡$jam_valid,X
## 
## ≡$jam_invalid(10.4)
Offender signatures must be similarly valid and ref-
erence work-reports with judgments and may not report
keys which are already in the punish-set:
∀(r,f,s)∈E
## C
## ∶
## ⋀
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
r∈ψ
## ′
## B
## ,
f∈k,
s∈
## ̄
## V
f
## ⟨X
## G
## ⌢r⟩
## (10.5)
∀(r,v,f,s)∈E
## F
## ∶
## ⋀
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
r∈ψ
## ′
## B
## ⇔r~∈ψ
## ′
## G
## ⇔v,
k∈k,
s∈
## ̄
## V
f
## ⟨X
v
## ⌢r⟩
## (10.6)
wherek={i
e
## Si∈λ∪κ}∖ψ
## O
VerdictsE
## V
must be ordered by report hash. Offender
signaturesE
## C
andE
## F
must each be ordered by the val-
idator’s Ed25519 key. There may be no duplicate report
hashes within the extrinsic, nor amongst any past reported
hashes. Formally:
## E
## V
=[(r,a,j)∈E
## V
## _
## _
r]
## (10.7)
## E
## C
=[(r,f,s)∈E
## C
## _
## _
f],E
## F
=[(r,v,f,s)∈E
## F
## _
## _
f](10.8)
{rS (r,a,j)∈E
## V
## }⫰ψ
## G
## ∪ψ
## B
## ∪ψ
## W
## (10.9)
The judgments of all verdicts must be ordered by val-
idator index and there may be no duplicates:
(10.10)∀(r,a,j)∈E
## V
## ∶j=[(v,i,s)∈j
## _
## _
i]
We definevto derive from the sequence of verdicts
introduced in the block’s extrinsic, containing only the
report hash and the sum of positive judgments. We re-
quire this total to be either exactly two-thirds-plus-one,
zero or one-third of the validator set indicating, respec-
tively, that the report is good, that it’s bad, or that it’s
wonky.
## 11
## Formally:
v∈⟦(H,{0,⌊
## 1
## ~3V⌋,⌊
## 2
## ~3V⌋+1})⟧(10.11)
v=
## ⎡
## ⎢
## ⎢
## ⎢
## ⎢
## ⎣
## ⎛
## ⎝
r,
## ∑
## (v,i,s)∈j
v
## ⎞
## ⎠
## R
## R
## R
## R
## R
## R
## R
## R
## R
## R
## R
## R
## (r,a,j)<
## −E
## V
## ⎤
## ⎥
## ⎥
## ⎥
## ⎥
## ⎦
## (10.12)
## 11
This requirement may seem somewhat arbitrary, but these happen to be the decision thresholds for our three possible actions and
are acceptable since the security assumptions include the requirement that at least two-thirds-plus-one validators are live (Jeff Burdges,
Cevallos, et al.
2024discusses the security implications in depth).

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202519
There are some constraints placed on the composition
of this extrinsic: any verdict containing solely valid judg-
ments implies the same report having at least one valid
entry in the faults sequenceE
## F
. Any verdict containing
solely invalid judgments implies the same report having at
least two valid entries in the culprits sequenceE
## C
## . For-
mally:
## ∀(r,⌊
## 2
~3V⌋+1)∈v∶ ∃(r,...)∈E
## F
## (10.13)
∀(r,0)∈v∶S{ (r,...)∈E
## C
## }S≥2(10.14)
We clear any work-reports which we judged as uncer-
tain or invalid from their core:
## (10.15)
∀c∈N
## C
## ∶ρ
## †
## [c]=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## ∅if
## 
## H
## 
ρ[c]
r
## 
## ,t
## 
## ∈v,t<⌊
## 2
## ~3V⌋
ρ[c]otherwise
The state’s good-set, bad-set and wonky-set assimi-
late the hashes of the reports from each verdict. Finally,
the punish-set accumulates the keys of any validators who
have been found guilty of offending. Formally:
ψ
## ′
## G
## ≡ψ
## G
∪{rS (r,⌊
## 2
~3V⌋+1)∈v}
## (10.16)
ψ
## ′
## B
## ≡ψ
## B
∪{rS (r,0)∈v}
## (10.17)
ψ
## ′
## W
## ≡ψ
## W
∪{rS (r,⌊
## 1
~3V⌋)∈v}
## (10.18)
ψ
## ′
## O
## ≡ψ
## O
∪{fS (f,...)∈E
## C
}∪{fS (f,...)∈E
## F
## }
## (10.19)
10.3.Header.The offenders markers must contain ex-
actly the keys of all new offenders, respectively. Formally:
## H
## O
≡[fS (f,...)<
## −E
## C
]⌢[fS (f,...)<−E
## F
## ](10.20)
11.Reporting and Assurance
Reporting and assurance are the two on-chain pro-
cesses we do to allow the results of in-core computation
to make their way into the state of service accounts,δ.
Awork-package, which comprises severalwork-items, is
transformed by validators acting asguarantorsinto its cor-
respondingwork-report, which similarly comprises several
work-digestsand then presented on-chain within theguar-
anteesextrinsic. At this point, the work-package is erasure
coded into a multitude of segments and each segment dis-
tributed to the associated validator who then attests to its
availability through anassuranceplaced on-chain. After
enough assurances the work-report is consideredavailable,
and the work-digests transform the state of their associ-
ated service by virtue of accumulation, covered in section
- The report may also betimed-out, implying it may be
replaced by another report without accumulation.
From the perspective of the work-report, therefore,
the guarantee happens first and the assurance after-
wards. However, from the perspective of a block’s state-
transition, the assurances are best processed first since
each core may only have a single work-report pending its
package becoming available at a time. Thus, we will first
cover the transition arising from processing the availability
assurances followed by the work-report guarantees. This
synchroneity can be seen formally through the require-
ment of an intermediate stateρ
## ‡
, utilized later in equation
## 11.29.
11.1.State.The state of the reporting and availability
portion of the protocol is largely contained withinρ, which
tracks the work-reports which have been reported but are
not yet known to be available to a super-majority of val-
idators, together with the time at which each was re-
ported. As mentioned earlier, only one report may be
assigned to a core at any given time. Formally:
## (11.1)ρ∈⟦
## ⎧
## ⎩
r∈R, t∈N
## T
## ⎫
## ⎭
## ?⟧
## C
As usual, intermediate and posterior values (ρ
## †
## ,ρ
## ‡
## ,ρ
## ′
## )
are held under the same constraints as the prior value.
11.1.1.Work Report.A work-report, of the setR, is de-
fined as a tuple of the work-package specification,s; the
refinement context,c; the core-index (i.e. on which the
work is done),c; as well as the authorizer hashaand
tracet; a segment-root lookup dictionaryl; the gas con-
sumed during the Is-Authorized invocation,g; and finally
the work-digestsdwhich comprise the results of the eval-
uation of each of the items in the package together with
some associated data. Formally:
## (11.2)R≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
s∈Y,c∈C, c∈N
## C
, a∈H,t∈B,
l∈jH→Ho,d∈⟦D⟧
## 1∶I
, g∈N
## G
## ⎫
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎭
We limit the sum of the number of items in the
segment-root lookup dictionary and the number of pre-
requisites toJ=8:
## (11.3)
∀r∈R∶Sr
l
S+S(r
c
## )
p
## S≤J
11.1.2.Refinement Context.Arefinement context, de-
noted by the setC, describes the context of the chain at
the point that the report’s corresponding work-package
was evaluated. It identifies two historical blocks, thean-
chor, header hashaalong with its associated posterior
state-rootsand accumulation output log super-peakb;
and thelookup-anchor, header hashland of timeslott.
Finally, it identifies the hash of any prerequisite work-
packagesp. Formally:
## (11.4)C≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
a∈H, s∈H,  b∈H,
l∈H, t∈N
## T
,p∈{[H]}
## ⎫
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎭
11.1.3.Availability.We define the set ofavailability spec-
ifications,Y, as the tuple of the work-package’s hashp, an
auditable work bundle lengthl(see section
14.4.1for more
clarity on what this is), together with an erasure-rootu,
a segment-rooteand segment-countn. Work-results in-
clude this availability specification in order to ensure they
are able to correctly reconstruct and audit the purported
ramifications of any reported work-package. Formally:
## Y≡
## ⎧
## ⎩
p∈H, l∈N
## L
, u∈H, e∈H, n∈N
## ⎫
## ⎭
## (11.5)
Theerasure-root(u) is the root of a binary Merkle
tree which functions as a commitment to all data required
for the auditing of the report and for use by later work-
packages should they need to retrieve any data yielded. It
is thus used by assurers to verify the correctness of data
they have been sent by guarantors, and it is later verified
as correct by auditors. It is discussed fully in section
## 14.
Thesegment-root(e) is the root of a constant-depth,
left-biased and zero-hash-padded binary Merkle tree com-
mitting to the hashes of each of the exported segments
of each work-item. These are used by guarantors to ver-
ify the correctness of any reconstructed segments they are

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202520
called upon to import for evaluation of some later work-
package. It is also discussed in section14.
11.1.4.Work Digest.We finally come to define awork-
digest,D, which is the data conduit by which services’
states may be altered through the computation done
within a work-package.
## (11.6)D≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
s∈N
## S
, c∈H, y∈H, g∈N
## G
,l∈B∪E,
u∈N
## G
, i∈N, x∈N, z∈N, e∈N
## ⎫
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎭
Work-digests are a tuple comprising several items.
Firstlys, the index of the service whose state is to be
altered and thus whose refine code was already executed.
We include the hash of the code of the service at the time
of being reportedc, which must be accurately predicted
within the work-report according to equation11.42.
Next, the hash of the payload (y) within the work item
which was executed in the refine stage to give this result.
This has no immediate relevance, but is something pro-
vided to the accumulation logic of the service. We follow
with the gas limitgfor executing this item’s accumulate.
There is the workresult, the output blob or error of
the execution of the code,l, which may be either an octet
sequence in case it was successful, or a member of the set
E, if not. This latter set is defined as the set of possible
errors, formally:
## (11.7)E∈
## 
## ∞,☇,⊚,⊖,BAD,BIG
## 
The first two are special values concerning execution of
the virtual machine,∞denoting an out-of-gas error and
☇denoting an unexpected program termination. Of the
remaining four, the first indicates that the number of ex-
ports made was invalidly reported, the second that the
size of the digest (refinement output) would cross the ac-
ceptable limit, the third indicates that the service’s code
was not available for lookup in state at the posterior state
of the lookup-anchor block. The fourth indicates that the
code was available but was beyond the maximum size al-
lowedW
## C
## .
Finally, we have five fields describing the level of activ-
ity which this workload imposed on the core in bringing
the result to bear. We includeuthe actual amount of gas
used during refinement;iandethe number of segments
imported from, and exported into, the D
## 3
L respectively;
andxandzthe number of, and total size in octets of, the
extrinsics used in computing the workload. See section
## 14
for more information on the meaning of these values.
In order to ensure fair use of a block’s extrinsic space,
work-reports are limited in the maximum total size of the
successful refinement output blobs together with the au-
thorizer trace, effectively limiting their overall size:
∀r∈R∶Sr
t
## S+
## ∑
d∈r
d
## ∩B
## Sd
l
## S≤W
## R
## (11.8)
## W
## R
## ≡48⋅2
## 10
## (11.9)
11.2.Package Availability Assurances.We first de-
fineρ
## ‡
, the intermediate state to be utilized next in sec-
tion
11.4as well asR, the set of available work-reports,
which will we utilize later in section
- Both require the
integration of information from the assurances extrinsic
## E
## A
## .
11.2.1.The Assurances Extrinsic.The assurances extrin-
sic is a sequence ofassurancevalues, at most one per val-
idator. Each assurance is a sequence of binary values (i.e.
a bitstring), one per core, together with a signature and
the index of the validator who is assuring. A value of1
(or⊺, if interpreted as a Boolean) at any given index im-
plies that the validator assures they are contributing to
its availability.
## 12
## Formally:
## E
## A
## ∈
## C
## ⎧
## ⎩
a∈H, f∈b
## C
, v∈N
## V
, s∈
## ̄
## V
## ⎫
## ⎭
## H
## ∶V
## (11.10)
The assurances must all be anchored on the parent and
ordered by validator index:
∀a∈E
## A
## ∶a
a
## =H
## P
## (11.11)
∀i∈{1...SE
## A
## S }∶E
## A
## [i−1]
v
## <E
## A
## [i]
v
## (11.12)
The signature must be one whose public key is that
of the validator assuring and whose message is the seri-
alization of the parent hashH
## P
and the aforementioned
bitstring:
∀a∈E
## A
## ∶a
s
## ∈
## ̄
## V
κ[a
v
## ]
e
## ⟨X
## A
## ⌢H(E(H
## P
## ,a
f
## ))⟩
## (11.13)
## X
## A
## ≡$jam_available(11.14)
A bit may only be set if the corresponding core has a
report pending availability on it:
(11.15)∀a∈E
## A
,c∈N
## C
## ∶a
f
## [c]⇒ρ
## †
## [c]≠∅
11.2.2.Available Reports.A work-report is said to be-
comeavailableif and only if there are a clear
## 2
## ~3super-
majority of validators who have marked its core as set
within the block’s assurance extrinsic. Formally, we de-
fine the sequence of newly available work-reportsRas:
## R≡
## ⎡
## ⎢
## ⎢
## ⎢
## ⎢
## ⎣
ρ
## †
## [c]
r
## R
## R
## R
## R
## R
## R
## R
## R
## R
## R
## R
c<
## −N
## C
## ,
## ∑
a∈E
## A
a
f
## [c]>
## 2
## ~3V
## ⎤
## ⎥
## ⎥
## ⎥
## ⎥
## ⎦
## (11.16)
This value is utilized in the definition of bothδ
## ′
andρ
## ‡
which we will define presently as equivalent toρ
## †
except
for the removal of items which are either now available or
have timed out:
∀c∈N
## C
## ∶ρ
## ‡
## [c]≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## ∅ifρ[c]
r
## ∈R∨H
## T
## ≥ρ
## †
## [c]
t
## +U
ρ
## †
## [c]otherwise
## (11.17)
11.3.Guarantor Assignments.Every block, each core
has three validators uniquely assigned to guarantee work-
reports for it. This is borne out withV=1,023validators
andC=341cores, since
## V
~C=3. The core index assigned
to each of the validators, as well as the validators’ keys
are denoted byM:
## (11.18)M∈
## ⎧
## ⎩
## ⟦N
## C
## ⟧
## V
## ,⟦K⟧
## V
## ⎫
## ⎭
We determine the core to which any given validator is
assigned through a shuffle using epochal entropy and a
periodic rotation to help guard the security and liveness
of the network. We useη
## 2
for the epochal entropy rather
thanη
## 1
to avoid the possibility of fork-magnification where
uncertainty about chain state at the end of an epoch could
give rise to two established forks before it naturally re-
solves.
We define the permute functionP, the rotation func-
tionRand finally the guarantor assignmentsMas follows:
R(c,n)≡[(x+n)modCSx<
## −c](11.19)
## 12
This is a “soft” implication since there is no consequence on-chain if dishonestly reported. For more information on this implication
see section
## 16.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202521
P(e,t)≡RF
## C⋅i
## V
 Vi<−N
## V
## ,e,
tmodE
## R
## (11.20)
## M≡
## 
## P(η
## ′
## 2
## ,τ
## ′
),Φ(κ
## ′
## )
## 
## (11.21)
We also defineM
## ∗
, which is equivalent to the valueM
as it would have been under the previous rotation:
## (11.22)
let(e,k)=
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## (η
## ′
## 2
## ,κ
## ′
## )if
τ
## ′
## −R
## E
## =
τ
## ′
## E
## 
## (η
## ′
## 3
## ,λ
## ′
## )otherwise
## M
## ∗
## ≡
## 
## P(e,τ
## ′
−R),Φ(k)
## 
11.4.Work Report Guarantees.We begin by defin-
ing the guarantees extrinsic,E
## G
, a series ofguarantees,
at most one for each core, each of which is a tuple of a
work-report, a credentialaand its corresponding timeslot
t. The core index of each guarantee must be unique and
guarantees must be in ascending order of this. Formally:
## E
## G
## ∈
## C
## ⎧
## ⎪
## ⎩
r∈R, t∈N
## T
, a∈
## C
## ⎧
## ⎩
## N
## V
## ,
## ̄
## V
## ⎫
## ⎭
## H
## 2∶3
## ⎫
## ⎪
## ⎭
## H
## ∶C
## (11.23)
## E
## G
=[g∈E
## G
## _
## _
## (g
r
## )
c
## ](11.24)
The credential is a sequence of two or three tuples of a
unique validator index and a signature. Credentials must
be ordered by their validator index:
∀g∈E
## G
## ∶g
a
## =[(v,s)∈g
a
## _
## _
v](11.25)
The signature must be one whose public key is that of
the validator identified in the credential, and whose mes-
sage is the serialization of the hash of the work-report.
The signing validators must be assigned to the core in
question in either this blockMif the timeslot for the
guarantee is in the same rotation as this block’s timeslot,
or in the most recent previous set of assignments,M
## ∗
## :
∀(r,t,a)∈E
## G
## ,
## ∀(v,s)∈a
## ∶
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
s∈
## ̄
## V
## (k
v
## )
e
## ⟨X
## G
⌢H(r)⟩
c
v
## =r
c
## ∧R(

τ
## ′
## ~R
## 
## −1)≤t≤τ
## ′
k∈G⇔∃(r,t,a)∈E
## G
## ,∃(v,s)∈a∶k=(k
v
## )
e
where(c,k)=
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## Mif
τ
## ′
## R
## =
t
## R
## 
## M
## ∗
otherwise
## (11.26)
## X
## G
## ≡$jam_guarantee(11.27)
We note that the Ed25519 key of each validator whose
signature is in a credential is placed in thereporterssetG.
This is utilized by the validator activity statistics book-
keeping system section13.
We denoteIto be the set of work-reports in the present
extrinsicE:
letI={g
r
Sg∈E
## G
## }
## (11.28)
No reports may be placed on cores with a report pend-
ing availability on it. A report is valid only if the autho-
rizer hash is present in the authorizer pool of the core on
which the work is reported. Formally:
## (11.29)
∀r∈I∶ρ
## ‡
## [r
c
## ]=∅ ∧r
a
## ∈α[r
c
## ]
We require that the gas allotted for accumulation of
each work-digest in each work-report respects its service’s
minimum gas requirements. We also require that all work-
reports’ total allotted accumulation gas is no greater than
the overall gas limitG
## A
## :
## (11.30)
∀r∈I∶
## ∑
d∈r
d
## (d
g
## )≤G
## A
## ∧ ∀d∈r
d
## ∶d
g
## ≥δ[d
s
## ]
g
11.4.1.Contextual Validity of Reports.For convenience,
we define two equivalencesxandpto be, respectively,
the set of all contexts and work-package hashes within
the extrinsic:
## (11.31)letx≡{r
c
Sr∈I},p≡{ (r
s
## )
p
Sr∈I}
There must be no duplicate work-package hashes (i.e.
two work-reports of the same package). Therefore, we
require the cardinality ofpto be the length of the work-
report sequenceI:
(11.32)SpS=SIS
We require that the anchor block be within the lastH
blocks and that its details be correct by ensuring that it
appears within our most recent blocksβ
## †
## H
## :
## ∀x∈x∶ ∃y∈β
## †
## H
## ∶x
a
## =y
h
## ∧x
s
## =y
s
## ∧x
b
## =y
b
## (11.33)
We require that each lookup-anchor block be within
the lastLtimeslots:
## ∀x∈x∶x
t
## ≥H
## T
## −L(11.34)
We also require that we have a record of it; this is one of
the few conditions which cannot be checked purely with
on-chain state and must be checked by virtue of retain-
ing the series of the lastLheaders as the ancestor setA.
Since it is determined through the header chain, it is still
deterministic and calculable. Formally:
∀x∈x∶ ∃h∈A∶h
## T
## =x
t
∧H(h)=x
l
## (11.35)
We require that the work-package of the report not be
the work-package of some other report made in the past.
We ensure that the work-package not appear anywhere
within our pipeline. Formally:
letq={ (r
s
## )
p
## S (r,d)∈
## Ì
ω}(11.36)
leta={ ((r
r
## )
s
## )
p
## Sr∈ρ,r≠∅}(11.37)
## ∀p∈p,p~∈
## ⋃
x∈β
## H
## K(x
p
## )∪
## ⋃
x∈ξ
x∪q∪a(11.38)
We require that the prerequisite work-packages, if
present, and any work-packages mentioned in the
segment-root lookup, be either in the extrinsic or in our
recent history.
∀r∈I,∀p∈(r
c
## )
p
∪K(r
l
## )∶
p∈p∪{xSx∈K(b
p
), b∈β
## H
## }
## (11.39)
We require that any segment roots mentioned in the
segment-root lookup be verified as correct based on our
recent work-package history and the present block:
letp={ (((g
r
## )
s
## )
p
## ↦((g
r
## )
s
## )
e
) Sg∈E
## G
## }
## (11.40)
∀r∈I∶r
l
## ⊆p∪
## ⋃
b∈β
## H
b
p
## (11.41)
(Note that these checks leave open the possibility of ac-
cepting work-reports in apparent dependency loops. We
do not consider this a problem: the pre-accumulation
stage effectively guarantees that accumulation never hap-
pens in these cases and the reports are simply ignored.)
Finally, we require that all work-digests within the ex-
trinsic predicted the correct code hash for their corre-
sponding service:
∀r∈I,∀d∈r
d
## ∶d
c
## =δ[d
s
## ]
c
## (11.42)

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202522
11.5.Transitioning for Reports.We defineρ
## ′
as be-
ing equivalent toρ
## ‡
, except where the extrinsic replaced
an entry. In the case an entry is replaced, the new value
includes the present timeτ
## ′
allowing for the value to be
replaced without respect to its availability once suﬀicient
time has elapsed (see equation11.29).
## (11.43)
∀c∈N
## C
## ∶ρ
## ′
## [c]≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
(r, t
## ▸
## ▸
τ
## ′
)if∃(r, t, a)∈E
## G
## ,r
c
## =c
ρ
## ‡
## [c]otherwise
This concludes the section on reporting and assurance.
We now have a complete definition ofρ
## ′
together withR
to be utilized in section12, describing the portion of the
state transition happening once a work-report is guaran-
teed and made available.
12.Accumulation
Accumulation may be defined as some function whose
arguments areRandδtogether with selected portions
of (at times partially transitioned) state and which yields
the posterior service stateδ
## ′
together with additional state
elementsι
## ′
## ,φ
## ′
andχ
## ′
## .
The proposition of accumulation is in fact quite sim-
ple: we merely wish to execute theAccumulatelogic of
the service code of each of the services which has at least
one work-digest, passing to it relevant data from said di-
gests together with useful contextual information. How-
ever, there are three main complications. Firstly, we must
define the execution environment of this logic and in par-
ticular the host functions available to it. Secondly, we
must define the amount of gas to be allowed for each ser-
vice’s execution. Finally, we must determine the nature
of transfers within Accumulate.
12.1.History and Queuing.Accumulation of a work-
report is deferred in the case that it has a not-yet-fulfilled
dependency and is cancelled entirely in the case of an in-
valid dependency. Dependencies are specified as work-
package hashes and in order to know which work-packages
have been accumulated already, we maintain a history of
what has been accumulated. This history,ξ, is suﬀiciently
large for an epoch worth of work-reports. Formally:
ξ∈⟦{[H]}⟧
## E
## (12.1)
## ©
ξ≡
## ⋃
x∈ξ
## (x)
## (12.2)
We also maintain knowledge of ready (i.e. available
and/or audited) but not-yet-accumulated work-reports in
the state itemω. Each of these were made available at
most one epoch ago but have or had unfulfilled dependen-
cies. Alongside the work-report itself, we retain its un-
accumulated dependencies, a set of work-package hashes.
## Formally:
ω∈
## C
## ⟦
## ⎧
## ⎩
## R,{[H]}
## ⎫
## ⎭
## ⟧
## H
## E
## (12.3)
The newly available work-reports,R, are partitioned
into two sequences based on the condition of having zero
prerequisite work-reports. Those meeting the condition,
## R
## !
, are accumulated immediately. Those not,
## R
## Q
, are for
queued execution. Formally:
## R
## !
≡[rSr<
−R,S(r
c
## )
p
## S=0∧r
l
## ={}](12.4)
## R
## Q
≡E([D(r) Sr<−R,S(r
c
## )
p
## S>0∨r
l
## ≠{}],
## ©
ξ)
## (12.5)
## D(r)≡(r,{ (r
c
## )
p
}∪K(r
l
## ))(12.6)
We define the queue-editing functionE, which is es-
sentially a mutator function for items such as those ofω,
parameterized by sets of now-accumulated work-package
hashes (those inξ). It is used to update queues of work-
reports when some of them are accumulated. Function-
ally, it removes all entries whose work-report’s hash is in
the set provided as a parameter, and removes any depen-
dencies which appear in said set. Formally:
## (12.7)E∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## ⟦
## ⎧
## ⎩
## R,
## {[
## H
## ]}
## ⎫
## ⎭
## ⟧,{[H]}
## ⎫
## ⎭
## →⟦
## ⎧
## ⎩
## R,{[H]}
## ⎫
## ⎭
## ⟧
## (r,x)↦(r,d∖x) W
## (r,d)<−r,
## (r
s
## )
p
## ~∈x

We further define the accumulation priority queue
functionQ, which provides the sequence of work-reports
which are able to be accumulated given a set of not-yet-
accumulated work-reports and their dependencies.
## (12.8)Q∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⟦
## ⎧
## ⎩
## R,{[H]}
## ⎫
## ⎭
## ⟧→⟦R⟧
r↦
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## []ifg=[]
g⌢Q(E(r,P(g)))otherwise
whereg=[rS (r,{})<
## −r]
Finally, we define the mapping function
## P
which ex-
tracts the corresponding work-package hashes from a set
of work-reports:
## (12.9)P∶
## {[R]}→{[H]}
r↦{ (r
s
## )
p
## Sr∈r}
We may now define the sequence of accumulatable
work-reports in this block asR
## ∗
## :
letm=H
## T
modE(12.10)
## R
## ∗
## ≡R
## !
⌢Q(q)
## (12.11)
whereq=E(
## Ï
ω
m...
## ⌢
## Ï
ω
## ...m
## ⌢R
## Q
## ,P(R
## !
## ))(12.12)
12.2.Execution.We work with a limited amount of gas
per block and therefore may not be able to process all
items inR
## ∗
in a single block. There are two slightly an-
tagonistic factors allowing us to optimize the amount of
work-items, and thus work-reports, accumulated in a sin-
gle block:
Firstly, while we have a well-known gas-limit for each
work-item to be accumulated, accumulation may still re-
sult in a lower amount of gas used. Only after a work-item
is accumulated can it be known if it uses less gas than the
advertised limit. This implies a sequential execution pat-
tern.
Secondly, sincepvmsetup cannot be expected to be
zero-cost, we wish to amortize this cost over as many
work-items as possible. This can be done by aggregating
work-items associated with the same service into the same
pvminvocation. This implies a non-sequential execution
pattern.
We resolve this by defining a function∆
## +
which accu-
mulates work-reports sequentially, and which itself uti-
lizes a function∆
## ∗
which accumulates work-reports in
a non-sequential, service-aggregated manner. In all but
the first invocation of∆
## +
, we also integrate the effects of
anydeferred-transfersimplied by the previous round of ac-
cumulation, thus the accumulation function must accept
both the information contained in work-digests and that
of deferred-transfers.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202523
Rather than passing whole work-digests into accumu-
late, we extract the salient information from them and
combine with information implied by their work-reports.
We call this kind of combined value anoperand tuple,U.
Likewise, we denote the set characterizing adeferred trans-
ferasX, noting that a transfer includes a memo compo-
nentmofW
## T
=128octets, together with the service in-
dex of the senders, the service index of the receiverd,
the balance to be transferredaand the gas limitgfor the
transfer. Formally:
## U≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
p∈H, e∈H, a∈H, y∈H,
g∈N
## G
,t∈B,l∈B∪E
## ⎫
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎭
## (12.13)
## X≡
## ⎧
## ⎩
s∈N
## S
,d∈N
## S
,a∈N
## B
,m∈B
## W
## T
,g∈N
## G
## ⎫
## ⎭
## (12.14)
## I≡U∪X(12.15)
Note that the union of the two is theaccumulation in-
put,I.
Our formalisms continue by definingSas a character-
ization of (i.e. values capable of representing) state com-
ponents which are both needed and mutable by the ac-
cumulation process. This comprises the service accounts
state (as inδ), the upcoming validator keysι, the queue
of authorizersφand the privileges stateχ. Formally:
## (12.16)
## S≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
d∈jN
## S
→Ao,i∈⟦K⟧
## V
## ,q∈
## C
## ⟦H⟧
## Q
## H
## C
, m∈N
## S
## ,
a∈⟦N
## S
## ⟧
## C
, v∈N
## S
, r∈N
## S
,z∈jN
## S
## →N
## G
o
## ⎫
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎭
Finally, we defineBandU, the sets characterizing
service-indexed commitments to accumulation output and
service-indexed gas usage respectively:
## (12.17)
## B≡{[
## ⎧
## ⎩
## N
## S
## ,H
## ⎫
## ⎭
## ]}U≡⟦
## ⎧
## ⎩
## N
## S
## ,N
## G
## ⎫
## ⎭
## ⟧
We define the outer accumulation function∆
## +
which
transforms a gas-limit, a sequence of deferred transfers, a
sequence of work-reports, an initial partial-state and a dic-
tionary of services enjoying free accumulation, into a tuple
of the number of work-reports accumulated, a posterior
state-context, the resultant accumulation-output pairings
and the service-indexed gas usage:
## (12.18)
## ∆
## +
## ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## N
## G
,⟦X⟧,⟦R⟧,S,jN
## S
## →N
## G
o
## ⎫
## ⎭
## →
## ⎧
## ⎩
## N,S,B,U
## ⎫
## ⎭
## (g,t,r,e,f)↦
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## (0,e,{},[])ifn=0
## (i+j,e
## ′
## ,b
## ∗
## ∪b,u
## ∗
## ⌢u)o/w
wherei=max(N
SrS+1
## )∶
## ∑
r∈r
## ...i
## ,d∈r
d
## (d
g
## )≤g
andn=StS+i+SfS
and
## 
e
## ∗
## ,t
## ∗
## ,b
## ∗
## ,u
## ∗
## 
## =∆
## ∗
## (e,t,r
## ...i
## ,f)
and
## 
j,e
## ′
## ,b,u
## 
## =∆
## +
## (g
## ∗
## −
## ∑
## (s,u)∈u
## ∗
## (u),t
## ∗
## ,r
i...
## ,e
## ∗
## ,
## {})
andg
## ∗
## =g+
## ∑
t∈t
## (t
g
## )
We come to define the parallelized accumulation func-
tion∆
## ∗
which, with the help of the single-service accu-
mulation function∆
## 1
, transforms an initial state-context,
together with a sequence of deferred transfers, a se-
quence of work-reports and a dictionary of privileged
always-accumulate services, into a tuple of the poste-
rior state-context, the resultant deferred-transfers and
accumulation-output pairings, and the service-indexed gas
usage. Note that for the privileges we employ a func-
tionRwhich selects the service to which the manager ser-
vice changed, or if no change was made, then that which
the service itself changed to. This allows privileges to be
‘owned‘ and facilitates the removal of the manager service
which we see as a helpful possibility. Formally:
## (12.19)
## ∆
## ∗
## ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
S,⟦X⟧,⟦R⟧,jN
## S
## →N
## G
o
## ⎫
## ⎭
## →
## ⎧
## ⎩
## S,⟦X⟧,B,U
## ⎫
## ⎭
## (e,t,r,f)↦
## 
d
## ′
## ,i
## ′
## ,q
## ′
## ,m
## ′
## ,a
## ′
## ,v
## ′
## ,r
## ′
## ,z
## ′
## 
## ,
## Ì
t
## ′
## ,b,u
## 
where:
lets={d
s
## Sr∈r,d∈r
d
}∪K(f)∪{t
d
## St∈t}
## ∆(s)≡∆
## 1
## (e,t,r,f,s)
u=
## [(
s,∆(s)
u
## ) S
s
## <−s]
b={ (s,b) Ss∈s, b=∆(s)
y
, b≠∅}
t
## ′
## =[∆(s)
t
## Ss<
## −s]
d
## ′
=I((d∪n)∖m,
## ⋃
s∈s
## ∆(s)
p
## )
## (d,i,q,m,a,v,r,z)=e
e
## ∗
## =∆(m)
e
## 
m
## ′
## ,z
## ′
## 
## =e
## ∗
## (m,z)
∀c∈N
## C
## ∶a
## ′
c
=R(a
c
## ,(e
## ∗
a
## )
c
## ,((∆(a
c
## )
e
## )
a
## )
c
## )
v
## ′
=R(v,e
## ∗
v
## ,(∆(v)
e
## )
v
## )
r
## ′
=R(r,e
## ∗
r
## ,(∆(r)
e
## )
r
## )
i
## ′
## =(∆(v)
e
## )
i
∀c∈N
## C
## ∶q
## ′
c
## =((∆(a
c
## )
e
## )
q
## )
c
n=
## ⋃
s∈s
## ((∆(s)
e
## )
d
∖K(d∖{s}))
m=
## ⋃
s∈s
(K(d)∖K((∆(s)
e
## )
d
## ))
## (12.20)
## R(o,a,b)≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
bifa=o
aotherwise
AndIis the preimage integration function, which
transforms a dictionary of service states and a set of ser-
vice/blob pairs into a new dictionary of service states.
Preimage provisions into services which no longer exist or
whose relevant request is dropped are disregarded:
## I∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
jN
## S
→Ao,{[
## ⎧
## ⎩
## N
## S
## ,B
## ⎫
## ⎭
## ]}
## ⎫
## ⎭
→jN
## S
→Ao
## (d,p)↦d
## ′
whered
## ′
## =dexcept:
## ∀
## (
s,
i)
## ∈
p
## , Y
## (
d
## ,s,
i
## )
## ∶
d
## ′
## [s]
l
[(H(i),SiS)]=
## 
τ
## ′
## 
d
## ′
## [s]
p
[H(i)]=i
## (12.21)
## Y∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
jN
## S
→Ao,N
## S
## ,B
## ⎫
## ⎭
## →{,⊺}
## (d,s,i)↦
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
d[s]
l
[(H(i),SiS)]=[]ifs∈K(d)
## otherwise
## (12.22)
We note that while forming the union of all altered,
newly added service and newly removed indices, defined
in the above context asK(n)∪m, different services may
not each contribute the same index for a new, altered or

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202524
removed service. This cannot happen for the set of re-
moved and altered services since the code hash of remov-
able services has no known preimage and thus cannot ex-
ecute itself to make an alteration. For new services this
should also never happen since new indices are explicitly
selected to avoid such conflicts. In the unlikely event it
does happen, the block must be considered invalid.
The single-service accumulation function,∆
## 1
, trans-
forms an initial state-context, a sequence of deferred-
transfers, a sequence of work-reports, a dictionary of ser-
vices enjoying free accumulation (with the values indicat-
ing the amount of free gas) and a service index into an
alterations state-context, a sequence oftransfers, a pos-
sible accumulation-output, the actualpvmgas used and
a set of preimage provisions. This function wrangles the
work-digests of a particular service from a set of work-
reports and invokespvmexecution with said data:
## (12.23)O≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
e∈S,t∈⟦X⟧, y∈H?,
u∈N
## G
## ,p∈{[
## ⎧
## ⎩
## N
## S
## ,B
## ⎫
## ⎭
## ]}
## ⎫
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎭
## ∆
## 1
## ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## S,⟦X⟧,⟦R⟧,
jN
## S
## →N
## G
o,N
## S
## ⎫
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎭
## →O
(e,t,r,f,s)↦Ψ
## A
## (e,τ
## ′
## ,s,g,i
## T
## ⌢i
## U
## )
where:
g=U(f
s
## ,0)+
## ∑
t∈t,t
d
## =s
## (t
g
## )+
## ∑
r∈r,d∈r
d
## ,d
s
## =s
## (d
g
## )
i
## T
=[tSt<
## −t
## ,t
d
## =
s]
i
## U
## =
l
## ▸
## ▸
d
l
, g
## ▸
## ▸
d
g
, y
## ▸
## ▸
d
y
## ,t
## ▸
## ▸
r
t
## ,
e
## ▸
## ▸
## (r
s
## )
e
, p
## ▸
## ▸
## (r
s
## )
p
## ,a
## ▸
## ▸
r
a
##  W
r<−r,
d<−r
d
, d
s
## =s

## (12.24)
This draws upong, the gas limit implied by the selected
deferred-transfers, work-reports and gas-privileges.
12.3.Final State Integration.Given the result of the
top-level∆
## +
, we may define the posterior stateχ
## ′
## ,φ
## ′
and
ι
## ′
as well as the first intermediate state of the service-
accountsδ
## †
and the Accumulation Output Logθ
## ′
## :
letg=max
## 
## G
## T
## ,G
## A
## ⋅C+
## ∑
x∈V(χ
## Z
## )
## (x)
## 
ande=(d
## ▸
## ▸
δ,i
## ▸
## ▸
ι,q
## ▸
## ▸
φ,m
## ▸
## ▸
χ
## M
## ,a
## ▸
## ▸
χ
## A
## ,v
## ▸
## ▸
χ
## V
## ,r
## ▸
## ▸
χ
## R
## ,z
## ▸
## ▸
χ
## Z
## )
## 
n,e
## ′
## ,b,u
## 
## ≡∆
## +
(g,[],R
## ∗
## ,e,χ
## Z
## )
## (12.25)
θ
## ′
## ≡[(s,h)∈b]
## (12.26)
## d
## ▸
## ▸
δ
## †
## ,i
## ▸
## ▸
ι
## ′
## ,q
## ▸
## ▸
φ
## ′
## ,m
## ▸
## ▸
χ
## ′
## M
## ,a
## ▸
## ▸
χ
## ′
## A
## ,v
## ▸
## ▸
χ
## ′
## V
## ,r
## ▸
## ▸
χ
## ′
## R
## ,z
## ▸
## ▸
χ
## ′
## Z
## ≡e
## ′
## (12.27)
From this formulation, we also receiven, the total num-
ber of work-reports accumulated andu, the gas used in
the accumulation process for each service. We compose
S, our accumulation statistics, which is a mapping from
the service indices which were accumulated to the amount
of gas used throughout accumulation and the number of
work-items accumulated. Formally:
S∈jN
## S
## →
## ⎧
## ⎩
## N
## G
## ,N
## ⎫
## ⎭
o
## (12.28)
S≡{ (s↦(G(s),N(s))) SG(s)+N(s)≠0}(12.29)
whereG(s)≡
## ∑
## (s,u)∈u
## (u)
andN(s)≡
## T
d
## T
r<
## −R
## ∗
## ...n
## ,d<−r
d
## ,d
s
## =s
## T
The second intermediate stateδ
## ‡
may then be defined
with the last-accumulation record being updated for all
accumulated services:
δ
## ‡
## ≡
## 
s↦a
## ′
## 
## U (s↦a)∈δ
## †
## (12.30)
wherea
## ′
## =
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
aexcepta
## ′
a
## =τ
## ′
ifs∈K(S)
aotherwise
## (12.31)
We define the final state of the ready queue and the ac-
cumulated map by integrating those work-reports which
were accumulated in this block and shifting any from the
prior state with the oldest such items being dropped en-
tirely:
ξ
## ′
## E−1
## =P(R
## ∗
## ...n
## )(12.32)
∀i∈N
## E−1
## ∶ξ
## ′
i
## ≡ξ
i+1
## (12.33)
∀i∈N
## E
## ∶ω
## ′
## ↺
m−i
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## E(R
## Q
## ,ξ
## ′
## E−1
## )ifi=0
## []if1≤i<τ
## ′
## −τ
## E(ω
## ↺
m−i
## ,ξ
## ′
## E−1
## )ifi≥τ
## ′
## −τ
## (12.34)
12.4.Preimage Integration.After accumulation, we
must integrate all preimages provided in the lookup ex-
trinsic to arrive at the posterior account state. The lookup
extrinsic is a sequence of pairs of service indices and data.
These pairs must be ordered and without duplicates (equa-
tion12.36requires this). The data must have been so-
licited by a service but not yet provided in thepriorstate.
## Formally:
## E
## P
## ∈⟦
## ⎧
## ⎩
## N
## S
## ,B
## ⎫
## ⎭
## ⟧
## (12.35)
## E
## P
=[i∈E
## P
## _
## _
i](12.36)
∀(s,d)∈E
## P
∶Y(δ,s,d)
## (12.37)
We disregard, without prejudice, any preimages which
due to the effects of accumulation are no longer useful.
We defineδ
## ′
as the state after the integration of the still-
relevant preimages:
## (12.38)δ
## ′
=I(δ
## ‡
## ,E
## P
## )
13.Statistics
13.1.Validator Activity.The
## J
amchain does not ex-
plicitly issue rewards—we leave this as a job to be done
by the staking subsystem (in Polkadot’s case envisioned
as a system parachain—hosted without fees—in the cur-
rent imagining of a public
## J
amnetwork). However, much
as with validator punishment information, it is important
for the
## J
amchain to facilitate the arrival of information
on validator activity in to the staking subsystem so that
it may be acted upon.
Such performance information cannot directly cover all
aspects of validator activity; whereas block production,
guarantor reports and availability assurance can easily be
tracked on-chain,Grandpa,Beefyand auditing activity
cannot. In the latter case, this is instead tracked with val-
idator voting activity: validators vote on their impression
of each other’s efforts and a median may be accepted as
the truth for any given validator. With an assumption of
50% honest validators, this gives an adequate means of
oraclizing this information.
The validator statistics are made on a per-epoch basis
and we retain one record of completed statistics together
with one record which serves as an accumulator for the
present epoch. Both are tracked inπ, which is thus a

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202525
sequence of two elements, with the first being the accu-
mulator and the second the previous epoch’s statistics.
For each epoch we track a performance record for each
validator:
π≡(π
## V
## ,π
## L
## ,π
## C
## ,π
## S
## )(13.1)
## ⎧
## ⎩
π
## V
## ,π
## L
## ⎫
## ⎭
## ∈⟦
## ⎧
## ⎩
b∈N,t∈N,p∈N,d∈N,g∈N,a∈N
## ⎫
## ⎭
## ⟧
## 2
## V
## (13.2)
The six validator statistics we track are:
b:The number of blocks produced by the validator.
t:The number of tickets introduced by the valida-
tor.
p:The number of preimages introduced by the val-
idator.
d:The total number of octets across all preimages
introduced by the validator.
g
## :
The number of reports guaranteed by the valida-
tor.
a:The number of availability assurances made by
the validator.
The objective statistics are updated in line with their
description, formally:
lete=
τ
## E
,  e
## ′
## =
τ
## ′
## E
## (13.3)
## 
a,π
## ′
## L
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## (π
## V
## ,π
## L
## )ife
## ′
## =e
## ([(0,...,[0,...]),...],π
## V
## )otherwise
## (13.4)
∀v∈N
## V
## ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
π
## ′
## V
## [v]
b
## ≡a[v]
b
+(v=H
## I
## )
π
## ′
## V
## [v]
t
## ≡a[v]
t
## +
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## SE
## T
Sifv=H
## I
## 0otherwise
π
## ′
## V
## [v]
p
## ≡a[v]
p
## +
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## SE
## P
Sifv=H
## I
## 0otherwise
π
## ′
## V
## [v]
d
## ≡a[v]
d
## +
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## ∑
d∈E
## P
SdSifv=H
## I
## 0otherwise
π
## ′
## V
## [v]
g
## ≡a[v]
g
## +(κ
## ′
v
## ∈G)
π
## ′
## V
## [v]
a
## ≡a[v]
a
+(∃a∈E
## A
## ∶a
v
## =v)
## (13.5)
Note thatGis theReportersset, as defined in equation
## 11.26.
13.2.Cores and Services.The other two components of
statistics are the core and service activity statistics. These
are tracked only on a per-block basis unlike the validator
statistics which are tracked over the whole epoch.
π
## C
## ∈F
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
d∈N, p∈N, i∈N, x∈N,
z∈N, e∈N, l∈N, u∈N
## G
## ⎫
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎭
## K
## C
## (13.6)
π
## S
∈nN
## S
## →
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
p∈(N,N),  r∈(N,N
## G
## ),
i∈N, x∈N, z∈N, e∈N,
a∈(N,N
## G
## )
## ⎫
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎭
s
## (13.7)
The core statistics are updated using several intermedi-
ate values from across the overall state-transition function;
I, the incoming work-reports, as defined in
11.28andR,
the newly available work-reports, as defined in11.16. We
define the statistics as follows:
∀c∈N
## C
## ∶π
## ′
## C
## [c]≡
## ⎛
## ⎜
## ⎜
## ⎝
i
## ▸
## ▸
## R(c)
i
, x
## ▸
## ▸
## R(c)
x
, z
## ▸
## ▸
## R(c)
z
## ,
e
## ▸
## ▸
## R(c)
e
, u
## ▸
## ▸
## R(c)
u
, l
## ▸
## ▸
## L(c),
d
## ▸
## ▸
D(c), p
## ▸
## ▸
## ∑
a∈E
## A
a
f
## [c]
## ⎞
## ⎟
## ⎟
## ⎠
## (13.8)
whereR(c∈N
## C
## )≡
## ∑
d∈r
d
,r∈I,r
c
## =c
## (d
i
## ,d
x
## ,d
z
## ,d
e
## ,d
u
## ,)(13.9)
andL(c∈N
## C
## )≡
## ∑
r∈I,r
c
## =c
## (r
s
## )
l
## (13.10)
andD(c∈N
## C
## )≡
## ∑
r∈R,r
c
## =c
## (r
s
## )
l
## +W
## G
## ⌈(r
s
## )
n
## 65
## ~64⌉(13.11)
Finally, the service statistics are updated using the
same intermediate values as the core statistics, but with
a different set of calculations:
## ∀s∈s∶π
## ′
## S
## [s]≡
## ⎛
## ⎜
## ⎜
## ⎜
## ⎜
## ⎜
## ⎝
i
## ▸
## ▸
## R(s)
i
, x
## ▸
## ▸
## R(s)
x
, z
## ▸
## ▸
## R(s)
z
## ,
e
## ▸
## ▸
## R(s)
e
, r
## ▸
## ▸
(R(s)
n
,R(s)
u
## ),
p
## ▸
## ▸
## ∑
(s,d)∈E
## P
(1,SdS),
a
## ▸
## ▸
U(S[s],(0,0))
## ⎞
## ⎟
## ⎟
## ⎟
## ⎟
## ⎟
## ⎠
## (13.12)
wheres=s
## R
## ∪s
## P
## ∪K(S)(13.13)
ands
## R
## ={d
s
## Sd∈r
d
,r∈I}
## (13.14)
ands
## P
={sS∃x∶(s,x)∈E
## P
## }(13.15)
andR(s∈N
## S
## )≡
## ∑
d∈r
d
,r∈I,d
s
## =s
## (n
## ▸
## ▸
## 1,d
u
## ,d
i
## ,d
x
## ,d
z
## ,d
e
## )(13.16)
14.Work Packages and Work Reports
14.1.Honest Behavior.We have so far specified how
to recognize blocks for a correctly transitioning
## J
am
blockchain. Through defining the state transition func-
tion and a state Merklization function, we have also de-
fined how to recognize a valid header. While it is not
especially diﬀicult to understand how a new block may be
authored for any node which controls a key which would
allow the creation of the two signatures in the header, nor
indeed to fill in the other header fields, readers will note
that the contents of the extrinsic remain unclear.
We define not only correct behavior through the cre-
ation of correct blocks but alsohonest behavior, which in-
volves the node taking part in severaloff-chainactivities.
This does have analogous aspects withinYPEthereum,
though it is not mentioned so explicitly in said document:
the creation of blocks along with the gossiping and inclu-
sion of transactions within those blocks would all count as
off-chain activities for which honest behavior is helpful. In
## J
am’s case, honest behavior is well-defined and expected
of at least
## 2
~3of validators.
Beyond the production of blocks, incentivized honest
behavior includes:
●the guaranteeing and reporting of work-packages,
along with chunking and distribution of both the
chunks and the work-package itself, discussed in
section
## 15;
●assuring the availability of work-packages after
being in receipt of their data;
●determining which work-reports to audit, fetching
and auditing them, and creating and distributing
judgments appropriately based on the outcome of
the audit;
●submitting the correct amount of auditing work
seen being done by other validators, discussed in
section
## 13.
14.2.Segments and the Manifest.Our basic erasure-
coding segment size isW
## E
=684octets, derived from the
fact we wish to be able to reconstruct even should almost
two-thirds of our 1023 participants be malicious or inca-
pacitated, the 16-bit Galois field on which the erasure-code

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202526
is based and the desire to eﬀiciently support encoding data
of close to, but no less than, 4kb.
Work-packages are generally small to ensure guaran-
tors need not invest a lot of bandwidth in order to discover
whether they can get paid for their evaluation into a work-
report. Rather than having much data inline, they instead
referencedata through commitments. The simplest com-
mitments are extrinsic data.
Extrinsic data are blobs which are being introduced
into the system alongside the work-package itself gener-
ally by the work-package builder. They are exposed to the
Refine logic as an argument. We commit to them through
including each of their hashes in the work-package.
Work-packages have two other types of external data
associated with them: A cryptographic commitment to
eachimportedsegment and finally the number of segments
which areexported.
14.2.1.Segments, Imports and Exports.The ability to
communicate large amounts of data from one work-
package to some subsequent work-package is a key fea-
ture of the
## J
amavailability system. An export segment,
defined as the setJ, is an octet sequence of fixed length
## W
## G
=4104. It is the smallest datum which may individ-
ually be imported from—or exported to—the long-term
## D
## 3
L during the Refine function of a work-package. Being
an exact multiple of the erasure-coding piece size ensures
that the data segments of work-package can be eﬀiciently
placed in the D
## 3
L system.
## (14.1)J≡B
## W
## G
Exported segments are data which aregenerated
through the execution of the Refine logic and thus are a
side effect of transforming the work-package into a work-
report. Since their data is deterministic based on the exe-
cution of the Refine logic, we do not require any particular
commitment to them in the work-package beyond know-
ing how many are associated with each Refine invocation
in order that we can supply an exact index.
On the other hand, imported segments are segments
which were exported by previous work-packages. In order
for them to be easily fetched and verified they are ref-
erenced not by hash but rather the root of a Merkle tree
which includes any other segments introduced at the time,
together with an index into this sequence. This allows for
justifications of correctness to be generated, stored, in-
cluded alongside the fetched data and verified. This is
described in depth in the next section.
14.2.2.Data Collection and Justification.It is the task of
a guarantor to reconstitute all imported segments through
fetching said segments’ erasure-coded chunks from enough
unique validators.  Reconstitution alone is not enough
since corruption of the data would occur if one or more
validators provided an incorrect chunk. For this reason
we ensure that the import segment specification (a Merkle
root and an index into the tree) be a kind of cryptographic
commitment capable of having a justification applied to
demonstrate that any particular segment is indeed correct.
Justification data must be available to any node over
the course of its segment’s potential requirement.  At
around 350 bytes to justify a single segment, justification
data is too voluminous to have all validators store all data.
We therefore use the same overall availability framework
for hosting justification metadata as the data itself.
The guarantor is able to use this proof to justify to
themselves that they are not wasting their time on incor-
rect behavior. We do not force auditors to go through
the same process. Instead, guarantors build anAuditable
Work Package, and place this in the Auditdasystem.
This is the original work-package, its extrinsic data, its
imported data and a concise proof of correctness of that
imported data. This tactic routinely duplicates data be-
tween the D
## 3
L and the Auditda, however it is acceptable
in order to reduce the bandwidth cost for auditors who
must justify the correctness as cheaply as possible as au-
diting happens on average 30 times for each work-package
whereas guaranteeing happens only twice or thrice.
14.3.Packages and Items.We begin by defining a
work-package, of setP, and its constituentwork-items, of
setW. A work-package includes a simple blob acting as an
authorization tokenj, the index of the service which hosts
the authorization codeh, an authorization code hashu
and a configuration blobf, a contextcand a sequence of
work itemsw:
## (14.2)
## P≡
## ⎧
## ⎩
j∈B, h∈N
## S
, u∈H,f∈B,c∈C,w∈⟦W⟧
## 1∶I
## ⎫
## ⎭
A work item includes:sthe identifier of the service to
which it relates, the code hash of the service at the time
of reportingc(whose preimage must be available from the
perspective of the lookup anchor block), a payload blob
y, gas limits for Refinement and Accumulationg&a, and
the three elements of its manifest, a sequence of imported
data segmentsiwhich identify a prior exported segment
through an index and the identity of an exporting work-
package,x, a sequence of blob hashes and lengths to be
introduced in this block (and which we assume the valida-
tor knows) andethe number of data segments exported
by this work item.
## (14.3)
## W≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
s∈N
## S
,c∈H,y∈B,g∈N
## G
,a∈N
## G
,e∈N,
i∈
## C
## ⎧
## ⎪
## ⎩
## H∪(H
## ⊞
## ),N
## ⎫
## ⎪
## ⎭
## H
## ,x∈⟦
## ⎧
## ⎩
## H,N
## ⎫
## ⎭
## ⟧
## ⎫
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎭
Note that an imported data segment’s work-package is
identified through the union of setsHand a tagged vari-
antH
## ⊞
. A value drawn from the regularHimplies the
hash value is of the segment-root containing the export,
whereas a value drawn fromH
## ⊞
implies the hash value is
the hash of the exporting work-package. In the latter case
it must be converted into a segment-root by the guaran-
tor and this conversion reported in the work-report for
on-chain validation.
We limit the total number of exported items toW
## X
## =
3072, the total number of imported items toW
## M
## =3072,
and the total number of extrinsics toT=128:
## (14.4)
∀p∈P∶
## ∑
w∈p
w
w
e
## ≤W
## X
## ∧
## ∑
w∈p
w
## Sw
i
## S≤W
## M
## ∧
## ∑
w∈p
w
## Sw
x
## S≤T
We make an assumption that the preimage to each ex-
trinsic hash in each work-item is known by the guarantor.
In general this data will be passed to the guarantor along-
side the work-package.
We limit the total size of the auditablework-bundle,
containing the work-package, import and extrinsic items,
together with all payloads, the authorizer configuration

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202527
and the authorization token to around 13.6mb. This limit
allows 2mb/s/core D
## 3
L imports, and thus a full comple-
ment of 3,072 imports, assuming no extrinsics, 64 bytes
for each of the authorization token and trace, and a work-
item payload of 4kb:
∀p∈P∶Sp
j
S+Sp
f
## S+
## ∑
w∈p
w
S(w)≤W
## B
whereS(w∈W)≡Sw
y
S+Sw
i
## S⋅W
## F
## +
## ∑
## (h,l)∈w
x
l
## (14.5)
## W
## F
## ≡W
## G
## +32⌈log
## 2
## (W
## M
## )⌉(14.6)
## W
## B
## ≡W
## M
## ⋅W
## F
## +4096+64+64=13,791,360(14.7)
We limit the sums of each of the two gas limits to be
at most the maximum gas allocated to a core for the cor-
responding operation:
(14.8)∀p∈P∶
## ∑
w∈p
w
## (w
a
## )<G
## A
## ∧
## ∑
w∈p
w
## (w
g
## )<G
## R
Given the resultland gas useduof some work-item,
we define the item-to-digest functionCas:
## (14.9)
## C∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## W,B∪E,N
## G
## ⎫
## ⎭
## →D
## 
s,c,y,
a,e,i,x
## ,l,u↦
## ⎛
## ⎜
## ⎝
s, c, y
## ▸
## ▸
H(y), g
## ▸
## ▸
a,l, u,
i
## ▸
## ▸
SiS, e, x
## ▸
## ▸
SxS, z
## ▸
## ▸
## ∑
## (h,z)∈x
z
## ⎞
## ⎟
## ⎠
We define the work-package’s implied authorizer asp
a
## ,
the hash of the authorization code hash concatenated with
the configuration. We define the authorization code asp
u
and require that it be available at the time of the lookup
anchor block from the historical lookup of servicep
h
## . For-
mally:
## (14.10)
∀p∈P∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
p
a
≡H(p
u
## ⌢p
f
## )
## E(↕p
m
## ,p
u
)≡Λ(δ[p
h
## ],(p
c
## )
t
## ,p
u
## )
## (p
m
## ,p
u
## )∈
## ⎧
## ⎩
## B,B
## ⎫
## ⎭
(The historical lookup function,Λ, is defined in equa-
tion
## 9.7.)
14.3.1.Exporting.Any of a work-package’s work-items
mayexportsegments and asegments-rootis placed in the
work-report committing to these, ordered according to the
work-item which is exporting. It is formed as the root of a
constant-depth binary Merkle tree as defined in equation
## E.4.
Guarantors are required to erasure-code and distrib-
ute two data sets: one blob, the auditablebundlecon-
taining the encoded work-package, extrinsic data and
self-justifying imported segments which is placed in the
short-term Auditdastore; and a second set of exported-
segments data together with thePaged-Proofsmetadata.
Items in the first store are short-lived; assurers are ex-
pected to keep them only until finality of the block in
which the availability of the work-result’s work-package
is assured. Items in the second, meanwhile, are long-
lived and expected to be kept for a minimum of 28 days
(672 complete epochs) following the reporting of the work-
report. This latter store is referred to as theDistributed,
## Decentralized, Data Lakeor D
## 3
L owing to its large size.
We define the paged-proofs functionPwhich accepts
a series of exported segmentssand defines some series
of additional segments placed into the D
## 3
L via erasure-
coding and distribution. The function evaluates to pages
of hashes, together with subtree proofs, such that justi-
fications of correctness based on a segments-root may be
made from it:
## (14.11)P∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⟦J⟧→⟦J⟧
s↦
## 
## P
l
## (E(↕J
## 6
(s,i),↕L
## 6
## (s,i)))
## T
i<−N
## ⌈
SsS
## ~64⌉
## 
wherel=W
## G
14.4.Computation of Work-Report.We now come
to the work-report computation functionΞ. This forms
the basis for all utilization of cores on
## J
am. It accepts
some work-packagepfor some nominated corecand re-
sults in either an error∇or the work-report and series
of exported segments. This function is deterministic and
requires only that it be evaluated within eight epochs of
a recently finalized block thanks to the historical lookup
functionality. It can thus comfortably be evaluated by
any node within the auditing period, even allowing for
practicalities of imperfect synchronization. Formally:
## (14.12)
## Ξ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## P,N
## C
## ⎫
## ⎭
## →R
## (p,c)↦
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
∇ift~∈B
## ∶W
## R
## (
s,c
## ▸
## ▸
p
c
## ,c,a
## ▸
## ▸
p
a
## ,t,l,d,g
## )
otherwise
## Where:
## K(l)≡
## 
h
## T
w∈p
w
## ,
## 
h
## ⊞
## ,n
## 
## ∈w
i
## 
,SlS≤8
(t,g)=Ψ
## I
## (p,c)
## (d,
e)=
## T
## 
(C(p
w
## [j],r,u),e)
## T
(r,u,e)=I(p,j), j<−N
## Sp
w
## S
## 
## I(p,j)≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
⊖,u,[J
## 0
## ,J
## 0
## ,...]
## ...m
e
## 
ifSrS+z>W
## R
## 
⊚,u,[J
## 0
## ,J
## 0
## ,...]
## ...m
e
## 
otherwise ifSeS≠m
e
## 
r,u,[J
## 0
## ,J
## 0
## ,...]
## ...m
e
## 
otherwise ifr~∈B
## (r,u,e)otherwise
where(r,e,u)=Ψ
## R
(c,j,p,o,S
## #
## (p
w
## ),ℓ)
andh=H(p), m=p
w
## [j], ℓ=
## ∑
k<j
p
w
## [k]
e
andz=SoS+
## ∑
k<j,(r∈B,...)=I(p,k)
SrS
Note that we gracefully handle both the case where the
output size of the work output would take the work-report
beyond its acceptable size and where number of segments
exported by a work-item’s Refinement execution is incor-
rectly reported in the work-item’s export segment count.
In both cases, the work-package continues to be valid as
a whole, but the work-item’s exported segments are re-
placed by a sequence of zero-segments equal in size to the
export segment count and its output is replaced by an
error.
Initially we constrain the segment-root dictionaryl: It
should contain entries for all unique work-package hashes
of imported segments not identified directly via a segment-
root but rather through a work-package hash.
We immediately define the segment-root lookup func-
tionL, dependent on this dictionary, which collapses
a union of segment-roots and work-package hashes into
segment-roots using the dictionary:
## (14.13)
L(r∈H∪H
## ⊞
## )≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
rifr∈H
l[h]if∃h∈H∶r=h
## ⊞
In order to expect to be compensated for a work-report
they are building, guarantors must compose a value forl
to ensure not only the above but also a further constraint

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202528
that all pairs of work-package hashes and segment-roots
do properly correspond:
## (14.14)
∀(h↦e)∈l∶ ∃p,c∈P,N
## C
∶H(p)=h∧(Ξ(p,c)
s
## )
e
## =e
As long as the guarantor is unable to satisfy the above
constraints, then it should consider the work-package un-
able to be guaranteed. Auditors are not expected to pop-
ulate this but rather to reuse the value in the work-report
they are auditing.
The next term to be introduced,(t,g), is the autho-
rization trace, the result of the Is-Authorized function to-
gether with the amount of gas it used. The second term,
(d,e)is the sequence of results for each of the work-items
in the work-package together with all segments exported
by each work-item. The third definitionIperforms an or-
dered accumulation (i.e. counter) in order to ensure that
the Refine function has access to the total number of ex-
ports made from the work-package up to the current work-
item.
The above relies on two functions,SandXwhich, re-
spectively, define the import segment data and the extrin-
sic data for some work-item argumentw. We also define
J, which compiles justifications of segment data:
## (14.15)
X(w∈W)≡[dS (H(d),SdS)<−w
x
## ]
S(w∈W)≡[b[n] SM(b)=L(r),(r,n)<−w
i
## ]
J(w∈W)≡[↕J
## 0
(b,n) SM(b)=L(r),(r,n)<−w
i
## ]
We may then definesas the data availability specifi-
cation of the package using these three functions together
with the yet to be definedAvailability Specifierfunction
A(see section14.4.1):
(14.16)s=A(H(p),Ep,X
## #
## (p
w
## ),S
## #
## (p
w
## ),J
## #
## (p
w
## ),
## Ì
e)
Note that while the formulations ofSandJseem to
require (due to the inner termb) all segments exported
by all work-packages exporting a segment to be imported,
such a vast amount of data is not generally needed. In par-
ticular, each justification can be derived through a single
paged-proof. This reduces the worst case data fetching
for a guarantor to two segments for every one to be im-
ported. In the case that contiguously exported segments
are imported (which we might assume is a fairly common
situation), then a single proof-page should be suﬀicient to
justify many imported segments.
Also of note is the lack of length prefixes: only the
Merkle paths for the justifications have a length prefix.
All other sequence lengths are determinable through the
work package itself.
The Is-Authorized logic it references must be executed
first in order to ensure that the work-package warrants the
needed core-time. Next, the guarantor should ensure that
all segment-tree roots which form imported segment com-
mitments are known and have not expired. Finally, the
guarantor should ensure that they can fetch all preimage
data referenced as the commitments of extrinsic segments.
Once done, then imported segments must be recon-
structed. This process may in fact be lazy as the Refine
function makes no usage of the data until thefetchhost-
call is made. Fetching generally implies that, for each im-
ported segment, erasure-coded chunks are retrieved from
enough unique validators (342, including the guarantor)
and is described in more depth in appendix
H. (Since
we specify systematic erasure-coding, its reconstruction
is trivial in the case that the correct 342 validators are re-
sponsive.) Chunks must be fetched for both the data itself
and for justification metadata which allows us to ensure
that the data is correct.
Validators, in their role as availability assurers, should
index such chunks according to the index of the segments-
tree whose reconstruction they facilitate. Since the data
for segment chunks is so small at 12 octets, fixed com-
munications costs should be kept to a bare minimum. A
good network protocol (out of scope at present) will al-
low guarantors to specify only the segments-tree root and
index together with a Boolean to indicate whether the
proof chunk need be supplied. Since we assume at least
341 other validators are online and benevolent, we can
assume that the guarantor can computeSandJabove
with confidence, based on the general availability of data
committed to withs
## ♣
, which is specified below.
14.4.1.Availability Specifier.We define the availability
specifier functionA, which creates an availability spec-
ifier from the package hash, an octet sequence of the
audit-friendly work-package bundle (comprising the work-
package itself, the extrinsic data and the concatenated im-
port segments along with their proofs of correctness), and
the sequence of exported segments:
## (14.17)A∶
## ⎧
## ⎩
## H,B,⟦J⟧
## ⎫
## ⎭
## →Y
(p,b,s)↦(p, l
## ▸
## ▸
SbS, u, e
## ▸
## ▸
M(s), n
## ▸
## ▸
SsS)
whereu=M
## B
## 
## Ì
x
## T
x<
## −
## T
## 
b
## ♣
## ,s
## ♣
## 
andb
## ♣
## =H
## #
## 
## C
## ⌈
SbS
## ~W
## E
## ⌉
## (P
## W
## E
## (b))
## 
ands
## ♣
## =M
## #
## B
## 
## T
## C
## #
## 6
(s⌢P(s))
The paged-proofs functionP, defined earlier in equa-
tion
14.11, accepts a sequence of segments and returns a
sequence of paged-proofs suﬀicient to justify the correct-
ness of every segment. There are exactly⌈
## 1
## ~64⌉paged-
proof segments as the number of yielded segments, each
composed of a page of 64 hashes of segments, together with
a Merkle proof from the root to the subtree-root which in-
cludes those 64 segments.
The functionsMandM
## B
are the fixed-depth and sim-
ple binary Merkle root functions, defined in equations
## E.4
andE.3. The functionCis the erasure-coding function,
defined in appendixH.
AndPis the zero-padding function to take an octet
array to some multiple ofnin length:
## (14.18)P
n∈N
## 1...
## ∶
## B→B
k⋅n
x↦x⌢[0,0,...]
((SxS+n−1)modn)+1...n
Validators are incentivized to distribute each newly
erasure-coded data chunk to the relevant validator, since
they are not paid for guaranteeing unless a work-report
is considered to beavailableby a super-majority of val-
idators. Given our work-packagep, we should therefore
send the corresponding work-package bundle chunk and
exported segments chunks to each validator whose keys
are together with similarly corresponding chunks for im-
ported, extrinsic and exported segments data, such that
each validator can justify completeness according to the
work-report’serasure-root. In the case of a coming epoch
change, they may also maximize expected reward by dis-
tributing to the new validator set.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202529
We will see this function utilized in the next sections,
for guaranteeing, auditing and judging.
15.Guaranteeing
Guaranteeing work-packages involves the creation and
distribution of a correspondingwork-reportwhich requires
certain conditions to be met. Along with the report, a sig-
nature demonstrating the validator’s commitment to its
correctness is needed. With two guarantor signatures, the
work-report may be distributed to the forthcoming
## J
am
chain block author in order to be used in theE
## G
, which
leads to a reward for the guarantors.
We presume that in a public system, validators will be
punished severely if they malfunction and commit to a
report which does not faithfully represent the result ofΞ
applied on a work-package. Overall, the process is:
(1)Evaluation of the work-package’s authorization,
and cross-referencing against the authorization
pool in the most recent
## J
amchain state.
(2)Creation and publication of a work-package re-
port.
(3)Chunking of the work-package and each of its ex-
trinsic and exported data, according to the era-
sure codec.
(4)Distributing the aforementioned chunks across
the validator set.
(5)Providing the work-package, extrinsic and ex-
ported data to other validators on request is also
helpful for optimal network performance.
For any work-packagepwe are in receipt of, we may
determine the work-report, if any, it corresponds to for
the corecthat we are assigned to. When
## J
amchain state
is needed, we always utilize the chain state of the most
recent block.
For any guarantor of indexvassigned to corecand a
work-packagep, we define the work-reportrsimply as:
## (15.1)
r=Ξ(p,c)
Such guarantors may safely create and distribute the
payload(s,v). The componentsmay be created accord-
ing to equation11.26; specifically it is a signature using
the validator’s registered Ed25519 key on a payloadl:
(15.2)l=H(E(r))
To maximize profit, the guarantor should require the
work-digest meets all expectations which are in place dur-
ing the guarantee extrinsic described in section11.4. This
includes contextual validity and inclusion of the autho-
rization in the authorization pool. No doing so does not
result in punishment, but will prevent the block author
from including the package and so reduces rewards.
Advanced nodes may maximize the likelihood that their
reports will be includable on-chain by attempting to pre-
dict the state of the chain at the time that the report will
get to the block author. Naive nodes may simply use the
current chain head when verifying the work-report. To
minimize work done, nodes should make all such evalua-
tionspriorto evaluating theΨ
## R
function to calculate the
report’s work-results.
Once evaluated as a reasonable work-package to guar-
antee, guarantors should maximize the chance that their
work is not wasted by attempting to form consensus over
the core. To achieve this they should send the work-
package to any other guarantors on the same core which
they do not believe already know of it.
In order to minimize the work for block authors and
thus maximize expected profits, guarantors should at-
tempt to construct their core’s next guarantee extrinsic
from the work-report, core index and set of attestations
including their own and as many others as possible.
In order to minimize the chance of any block authors
disregarding the guarantor for anti-spam measures, guar-
antors should sign an average of no more than two work-
reports per timeslot.
16.Availability Assurance
Validators should issue a signed statement, called an
assurance, when they are in possession of all of their cor-
responding erasure-coded chunks for a given work-report
which is currently pending availability. For any work-
report to gain an assurance, there are two classes of data
a validator must have:
Firstly, their erasure-coded chunk for this report’s bun-
dle. The validity of this chunk can be trivially proven
through the work-report’s work-package erasure-root and
a Merkle-proof of inclusion in the correct location. The
proof should be included from the guarantor. This chunk
is needed to verify the work-report’s validity and com-
pleteness and need not be retained after the work-report
is considered audited. Until then, it should be provided
on request to validators.
Secondly, the validator should have in hand the cor-
responding erasure-coded chunk for each of the exported
segments referenced by thesegments root. These should
be retained for 28 days and provided to any validator on
request.
17.Auditing and Judging
The auditing and judging system is theoretically equiv-
alent to that inElves, introduced by Jeff Burdges, Ceval-
los, et al.
- For a full security analysis of the mecha-
nism, see this work. There is a difference in terminology,
where the termsbacking,approvalandinclusionthere re-
fer to our guaranteeing, auditing and accumulation, re-
spectively.
17.1.Overview.The auditing process involves each
node requiring themselves to fetch, evaluate and issue
judgment on a random but deterministic set of work-
reports from each
## J
amchain block in which the work-
report becomes available (i.e. fromR). Prior to any eval-
uation, a node declares and proves its requirement. At
specific common junctures in time thereafter, the set of
work-reports which a node requires itself to evaluate from
each block’sRmay be enlarged if any declared intentions
are not matched by a positive judgment in a reasonable
time or in the event of a negative judgment being seen.
These enlargement events are called tranches.
If all declared intentions for a work-report are matched
by a positive judgment at any given juncture, then the
work-report is consideredaudited. Once all of any given
block’s newly available work-reports are audited, then we
consider the block to beaudited. One prerequisite of a
node finalizing a block is for it to view the block as au-
dited. Note that while there will be eventual consensus on

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202530
whether a block is audited, there may not be consensus
at the time that the block gets finalized. This does not
affect the crypto-economic guarantees of this system.
In regular operation, no negative judgments will ul-
timately be found for a work-report, and there will be
no direct consequences of the auditing stage. In the un-
likely event that a negative judgment is found, then one
of several things happens; if there are still more than
## 2
## ~3V
positive judgments, then validators issuing negative judg-
ments may receive a punishment for time-wasting. If there
are greater than
## 1
~3Vnegative judgments, then the block
which includes the work-report is ban-listed. It and all
its descendants are disregarded and may not be built on.
In all cases, once there are enough votes, a judgment ex-
trinsic can be constructed by a block author and placed
on-chain to denote the outcome. See section10for details
on this.
All announcements and judgments are published to all
validators along with metadata describing the signed ma-
terial. On receipt of sure data, validators are expected to
update their perspective accordingly (later defined asJ
andA).
17.2.Data Fetching.For each work-report to be au-
dited, we use its erasure-root to request erasure-coded
chunks from enough assurers. From each assurer we fetch
three items (which with a good network protocol should be
done under a single request) corresponding to the work-
package super-chunks, the self-justifying imports super-
chunks and the extrinsic segments super-chunks.
We may validate the work-package reconstruction by
ensuring its hash is equivalent to the hash includes as
part of the work-package specification in the work-report.
We may validate the extrinsic segments through ensur-
ing their hashes are each equivalent to those found in the
relevant work-item.
Finally, we may validate each imported segment as a
justification must follow the concatenated segments which
allows verification that each segment’s hash is included in
the referencing Merkle root and index of the correspond-
ing work-item.
Exported segments need not be reconstructed in the
same way, but rather should be determined in the same
manner as with guaranteeing, i.e. through the execution
of the Refine logic.
All items in the work-package specification field of the
work-report should be recalculated from this now known-
good data and verified, essentially retracing the guaran-
tors steps and ensuring correctness.
17.3.Selection of Reports.Each validator shall per-
form auditing duties on each valid block received. Since
we are entering off-chain logic, and we cannot assume con-
sensus, we henceforth consider ourselves a specific valida-
tor of indexvand assume ourselves focused on some re-
cent blockBwith other terms corresponding to the state-
transition implied by that block, soρis said block’s prior
core-allocation,κis its prior validator set,His its header
&c. Practically, all considerations must be replicated for
all blocks and multiple blocks’ considerations may be un-
derway simultaneously.
We define the sequence of work-reports which we may
be required to audit asq, a sequence of length equal to
the number of cores, which functions as a mapping of core
index to a work-report pending which has just become
available, or∅if no report became available on the core.
## Formally:
q∈⟦R?⟧
## C
## (17.1)
q≡
ρ[c]
r
ifρ[c]
r
## ∈R
## ∅otherwise
¡ Wc<−N
## C
## (17.2)
We define our initial audit tranche in terms of a verifi-
able random quantitys
## 0
created specifically for it:
s
## 0
## ∈
## ∽
## V
## []
κ[v]
b
## ⟨X
## U
## ⌢Y(H
## V
## )⟩(17.3)
## X
## U
## =$jam_audit(17.4)
We may then definea
## 0
as the non-empty items to audit
through a verifiably random selection of ten cores:
a
## 0
## ={ (r,c) S (r,c)∈p
## ⋅⋅⋅+10
## ,r≠∅}(17.5)
wherep=F([(c,q
c
) Sc<−N
## C
],Y(s
## 0
## ))(17.6)
EveryA=8seconds following a new time slot, a new
tranche begins, and we may determine that additional
cores warrant an audit from us. Such items are defined
asa
n
wherenis the current tranche. Formally:
## (17.7)letn=
## T−P⋅H
## T
## A
## 
New tranches may contain items fromqstemming from
one of two reasons: either a negative judgment has been
received; or the number of judgments from the previous
tranche is less than the number of announcements from
said tranche. In the first case, the validator is always re-
quired to issue a judgment on the work-report. In the sec-
ond case, a new special-purposevrfmust be constructed
to determine if an audit and judgment is warranted from
us.
In all cases, we publish a signed statement of which
of the cores we believe we are required to audit (anan-
nouncement) together with evidence of thevrfsignature
to select them and the other validators’ announcements
from the previous tranche unmatched with a judgment in
order that all other validators are capable of verifying the
announcement.Publication of an announcement should be
taken as a contract to complete the audit regardless of any
future information.
Formally, for each tranchenwe ensure the announce-
ment statement is published and distributed to all other
validators along with our validator indexv, evidences
n
and all signed data. Validator’s announcement statements
must be in the setS:
## S≡
## ̄
## V
κ[v]
e
## ⟨
## X
## I
n⌢x
n
## ⌢H(H)⟩(17.8)
wherex
n
## =E({E
## 2
(c)⌢H(r) S (r,c)∈a
n
## })(17.9)
## X
## I
## =$jam_announce(17.10)
We defineA
n
as our perception of which validator is
required to audit each of the work-reports (identified by
their associated core) at tranchen. This comes from each
other validators’ announcements (defined above). It can-
not be correctly evaluated untilnis current. We have
absolute knowledge about our own audit requirements.
## A
n
## ∶R→{[N
## V
## ]}
## (17.11)
## (17.12)
We further defineJ
## ⊺
andJ
## 
to be the validator in-
dices who we know to have made respectively, positive
and negative, judgments mapped from each work-report’s

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202531
core. We don’t care from which tranche a judgment is
made.
## J
## {,⊺}
## ∶R→{[N
## V
## ]}(17.13)
We are able to definea
n
for tranches beyond the first
on the basis of the number of validators who we know are
required to conduct an audit yet from whom we have not
yet seen a judgment. It is possible that the late arrival
of information altersa
n
and nodes should reevaluate and
act accordingly should this happen.
We can thus definea
n
beyond the initial tranche
through a newvrfwhich acts upon the set ofno-show
validators.
## ∀n>0∶
s
n
## (r)∈
## ∽
## V
## []
κ[v]
b
## ⟨X
## U
## ⌢Y(H
## V
)⌢H(r)n⟩(17.14)
a
n
## ≡
## 
r
## T
## V
## 256
## F
## Y(s
n
## (r))
## 0
## <m
n
## ,r∈q,r≠∅
## 
## (17.15)
wherem
n
## =SA
n−1
(r)∖J
## ⊺
(r)S
We define our bias factorF=2, which is the expected
number of validators which will be required to issue a
judgment for a work-report given a single no-show in the
tranche before. Modeling by Jeff Burdges, Cevallos, et al.
2024shows that this is optimal.
Later audits must be announced in a similar fashion to
the first. If audit requirements lessen on the receipt of new
information (i.e. a positive judgment being returned for
a previousno-show), then any audits already announced
are completed and judgments published. If audit require-
ments raise on the receipt of new information (i.e. an addi-
tional announcement being found without an accompany-
ing judgment), then we announce the additional audit(s)
we will undertake.
Asnincreases with the passage of timea
n
becomes
known and defines our auditing responsibilities. We must
attempt to reconstruct all work-packages and their requi-
site data corresponding to each work-report we must au-
dit. This may be done through requesting erasure-coded
chunks from one-third of the validators. It may also be
short-cutted by asking a cooperative third party (e.g. an
original guarantor) for the preimages.
Thus, for any such work-reportrwe are assured we
will be able to fetch some candidate work-package encod-
ingF(r)which comes either from reconstructing erasure-
coded chunks verified through the erasure coding’s Merkle
root, or alternatively from the preimage of the work-
package hash. We decode this candidate blob into a work-
package.
In addition to the work-package, we also assume we are
able to fetch all manifest data associated with it through
requesting and reconstructing erasure-coded chunks from
one-third of validators in the same way as above.
We then attempt to reproduce the report on the core
to givee
n
, a mapping from cores to evaluations:
## (17.16)
## ∀(c,r)∈a
n
## ∶
e
n
## (c)⇔
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
r=Ξ(p,c)if∃p∈P∶E(p)=F(r)
## otherwise
Note that a failure to decode implies an invalid work-
report.
From this mapping the validator issues a set of judg-
mentsj
n
## :
j
n
## =
## 
## ̄
## S
κ[]
## 
## X
e
n
## (c)
⌢H(r)
##  T
## (c,r)∈a
n
## 
## (17.17)
All judgmentsj
## ∗
should be published to other valida-
tors in order that they build their view ofJand in the
case of a negative judgment arising, can form an extrinsic
forE
## D
## .
We consider a work-report as audited under two cir-
cumstances. Either, when it has no negative judgments
and there exists some tranche in which we see a positive
judgment from all validators who we believe are required
to audit it; or when we see positive judgments for it from
greater than two-thirds of the validator set.
## U(r)⇔
## ⋁
## 
## J
## 
(r)=∅ ∧ ∃n∶A
n
(r)⊂J
## ⊺
## (r)
## SJ
## ⊺
(r)S>
## 2
## ~3V
## (17.18)
Our blockBmay be considered audited, a condition
denotedU, when all the work-reports which were made
available are considered audited. Formally:
U⇔∀r∈R∶U(r)(17.19)
For any block we must judge it to be audited (i.e.
U=⊺) before we vote for the block to be finalized in
Grandpa. See section19for more information here.
Furthermore, we pointedly disregard chains which in-
clude the accumulation of a report which we know at least
## 1
~3of validators judge as being invalid. Any chains includ-
ing such a block are not eligible for authoring on. Thebest
block, i.e. that on which we build new blocks, is defined as
the chain with the most regular Safrole blocks which does
notcontain any such disregarded block. Implementation-
wise, this may require reversion to an earlier head or al-
ternative fork.
As a block author, we include a judgment extrinsic
which collects judgment signatures together and reports
them on-chain. In the case of a non-valid judgment (i.e.
one which is not two-thirds-plus-one of judgments con-
firming validity) then this extrinsic will be introduced in a
block in which accumulation of the non-valid work-report
is about to take place. The non-valid judgment extrin-
sic removes it from the pending work-reports,ρ. Refer to
section
10for more details on this.
18.Beefy Distribution
For each finalized blockBwhich a validator imports,
said validator shall make ablssignature on thebls12-381
curve, as defined by Hopwood et al.2020, aﬀirming the
Keccak hash of the block’s most recentBeefy mmr. This
should be published and distributed freely, along with the
signed material. These signatures may be aggregated in
order to provide concise proofs of finality to third-party
systems. The signing and aggregation mechanism is de-
fined fully by Jeff Burdges, Ciobotaru, et al.
## 2022.
Formally, letF
v
be the signed commitment of validator
indexvwhich will be published:
## F
v
## ≡
## BLS
## S
κ
## ′
## (X
## B
## ⌢last(β
## H
## )
b
## )(18.1)
## X
## B
## =$jam_beefy(18.2)
19.Grandpa and the Best Chain
Nodes take part in theGrandpaprotocol as defined
by Stewart and Kokoris-Kogia
## 2020.
We define the latest finalized block asB
## ♮
. All associ-
ated terms concerning block and state are similarly super-
scripted. We consider thebest block,B
## ♭
to be that which

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202532
is drawn from the set of acceptable blocks of the following
criteria:
●Has the finalized block as an ancestor.
●Contains no unfinalized blocks where we see an
equivocation (two valid blocks at the same times-
lot).
●Is considered audited.
## Formally:
## A(H
## ♭
## )∋H
## ♮
## (19.1)
## U
## ♭
## ≡⊺(19.2)
## ~∃H
## A
## ,H
## B
## ∶
## ⋀
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## H
## A
## ≠H
## B
## H
## A
## T
## =H
## B
## T
## H
## A
## ∈A(H
## ♭
## )
## H
## A
## ~∈A(H
## ♮
## )
## (19.3)
Of these acceptable blocks, that which contains the
most ancestor blocks whose author used a seal-key ticket,
rather than a fallback key should be selected as the best
head, and thus the chain on which the participant should
makeGrandpavotes.
Formally, we aim to selectB
## ♭
to maximize the valuem
where:
## (19.4)m=
## ∑
## H
## A
## ∈A
## ♭
## T
## A
The specific data to be voted on inGrandpashall be
the block header of the best block,B
## ♭
together with its
posteriorstate root,M
σ
## (σ
## ′
). The state root has no di-
rect relevance to theGrandpaprotocol, but is included
alongside the header during voting/signing into order to
ensure that systems utilizing the output ofGrandpaare
able to verify the most recent chain state as possible.
This implies that the posterior state must be known at
the time thatGrandpavoting occurs in order to finalize
the block. However, sinceGrandpais relied on primarily
for state-root verification it makes little sense to finalize a
block without an associated commitment to the posterior
state.
The posterior state only affectsGrandpavoting in so
much as votes for the same block hash but with different
associated posterior state roots are considered votes for
different blocks. This would only happen in the case of
a misbehaving node or an ambiguity in the present docu-
ment.
20.Discussion
20.1.Technical Characteristics.In total, with our
stated target of 1,023 validators and three validators per
core, along with requiring a mean of ten audits per val-
idator per timeslot, and thus 30 audits per work-report,
## J
amis capable of trustlessly processing and integrating
341 work-packages per timeslot.
We assume node hardware is a modern 16 corecpu
with 64gb ram, 8tbsecondary storage and 0.5gbe net-
working.
Our performance models assume a rough split ofcpu
time as follows:
## Proportion
## Audits
## 10
## ~16
## Merklization
## 1
## ~16
Block execution
## 2
## ~16
GrandpaandBeefy
## 1
## ~16
Erasure coding
## 1
## ~16
Networking & misc
## 1
## ~16
Estimates for network bandwidth requirements are as
follows:
Throughput,mb/slotTx  Rx
## Guaranteeing106  48
## Assuring144  13
## Auditing0   133
## Authoring53   87
GrandpaandBeefy4   4
## Total304 281
Implied bandwidth,mb/s387 357
Thus, a connection able to sustain 500mb/s should
leave a suﬀicient margin of error and headroom to serve
other validators as well as some public connections, though
the burstiness of block publication would imply validators
are best to ensure that peak bandwidth is higher.
Under these conditions, we would expect an overall
network-provided data availability capacity of 2pb, with
each node dedicating at most6tbto availability storage.
Estimates for memory usage are as follows:
gb
## Auditing20  2×10pvminstances
Block execution  2   1pvminstance
State cache40
## Misc2
## Total64
As a rough guide, each parachain has an average foot-
print of around 2mbin the Polkadot Relay chain; a 40gb
state would allow 20,000 parachains’ information to be
retained in state.
What might be called the “virtual hardware” of a
## J
am
core is essentially a regularcpucore executing at some-
where between 25% and 50% of regular speed for the
whole six-second portion and which may draw and pro-
vide 2mb/s average in general-purposei/oand utilize up
to 2gbinram. Thei/oincludes any trustless reads from
the
## J
amchain state, albeit in the recent past. This virtual
hardware also provides unlimited reads from a semi-static
preimage-lookup database.
Each work-package may occupy this hardware and ex-
ecute arbitrary code on it in six-second segments to create
some result of at most 48kb. This work-result is then en-
titled to 10ms on the same machine, this time with no
“external”i/o, but instead with full and immediate ac-
cess to the
## J
amchain state and may alter the service(s)
to which the results belong.
20.2.Illustrating Performance.In terms of pure pro-
cessing power, the
## J
ammachine architecture can deliver
extremely high levels of homogeneous trustless computa-
tion. However, the core model of
## J
amis a classic paral-
lelized compute architecture, and for solutions to be able
to utilize the architecture well they must be designed with

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202533
it in mind to some extent. Accordingly, until such use-
cases appear on
## J
amwith similar semantics to existing
ones, it is very diﬀicult to make direct comparisons to ex-
isting systems. That said, if we indulge ourselves with
some assumptions then we can make some crude compar-
isons.
20.2.1.Comparison to Polkadot.Polkadot is at present
capable of validating at most 80 parachains each doing one
second of native computation and 5mbofi/oin every six.
This corresponds to an aggregate compute performance
of around 13x nativecpuand a total 24-hour distributed
availability of around 67mb/s. Accumulation is beyond
Polkadot’s capabilities and so not comparable.
For comparison, in our basic models,
## J
amshould be
capable of attaining around 85x the computation load of
a single nativecpucore and a distributed availability of
## 682mb/s.
20.2.2.Simple Transfers.We might also attempt to
model a simple transactions-per-second amount, with each
transaction requiring a signature verification and the mod-
ification of two account balances. Once again, until there
are clear designs for precisely how this would work we must
make some assumptions. Our most naive model would be
to use the
## J
amcores (i.e. refinement) simply for trans-
action verification and account lookups. The
## J
amchain
would then hold and alter the balances in its state. This
is unlikely to give great performance since almost all the
neededi/owould be synchronous, but it can serve as a
basis.
A 12mbwork-package can hold around 96k transactions
at 128 bytes per transaction. However, a 48kbwork-result
could only encode around 6k account updates when each
update is given as a pair of a 4 byte account index and 4
byte balance, resulting in a limit of 3k transactions per
package, or 171ktpsin total. It is possible that the
eight bytes could typically be compressed by a byte or
two, increasing maximum throughput a little. Our ex-
pectations are that state updates, with highly parallelized
Merklization, can be done at between 500k and 1 million
reads/write per second, implying around 250k-350ktps,
depending on which turns out to be the bottleneck.
A more sophisticated model would be to use the
## J
am
cores for balance updates as well as transaction verifica-
tion. We would have to assume that state and the trans-
actions which operate on them can be partitioned between
work-packages with some degree of eﬀiciency, and that the
12mbof the work-package would be split between trans-
action data and state witness data. Our basic models
predict that a 32-bit account system paginated into2
## 10
accounts/page and 128 bytes per transaction could, as-
suming only around 1% of oraclized accounts were useful,
average upwards of 1.4mtpsdepending on partitioning
and usage characteristics. Partitioning could be done with
a fixed fragmentation (essentially sharding state), a ro-
tating partition pattern or a dynamic partitioning (which
would require specialized sequencing).
Interestingly, we expect neither model to be bot-
tlenecked in computation, meaning that transactions
could be substantially more sophisticated, perhaps with
more flexible cryptography or smart-contract functional-
ity, without a significant impact on performance.
20.2.3.Computation Throughput.Thetpsmetric does
not lend itself well to measuring distributed systems’ com-
putational performance, so we now turn to another slightly
more compute-focussed benchmark: theevm. The basic
YPEthereum network, now approaching a decade old, is
probably the best known example of general purpose de-
centralized computation and makes for a reasonable yard-
stick. It is able to sustain a computation andi/orate of
1.25M gas/sec, with a peak throughput of twice that. The
evmgas metric was designed to be a time-proportional
metric for predicting and constraining program execution.
Attempting to determine a concrete comparison topvm
throughput is non-trivial and necessarily opinionated ow-
ing to the disparity between the two platforms, includ-
ing word size, endianness, stack/register architecture and
memory model. However, we will attempt to determine a
reasonable range of values.
Evmgas does not directly translate into native execu-
tion as it also combines state reads and writes as well as
transaction input data, implying it is able to process some
combination of up to 595 storage reads, 57 storage writes
and 1.25M computation-gas as well as 78kbinput data in
each second, trading one against the other.
## 13
We cannot
find any analysis of the typical breakdown between storage
i/oand pure computation, so to make a very conservative
estimate, we assume it does all four. In reality, we would
expect it to be able to do on average
## 1
/4of each.
Our experiments
## 14
show that on modern, high-end con-
sumer hardware with a high-qualityevmimplementation,
we can expect somewhere between 100 and 500 gas/μs in
throughput on pure-compute workloads (we specifically
utilized Odd-Product, Triangle-Number and several im-
plementations of the Fibonacci calculation). To make a
conservative comparison topvm, we propose transpilation
of theevmcode intopvmcode and then re-execution of
it under the Polkavmprototype.
## 15
To help estimate a reasonable lower-bound ofevm
gas/μs, e.g. for workloads which are more memory and
i/ointensive, we look toward real-world permissionless
deployments of theevmand see that the Moonbeam
network, after correcting for the slowdown of execut-
ing within the recompiled WebAssembly platform on the
somewhat conservative Polkadot hardware platform, im-
plies a throughput of around 100 gas/μs. We therefore
assert that in terms of computation, 1μs approximates to
around 100-500evmgas on modern high-end consumer
hardware.
## 16
## 13
The latest “proto-danksharding” changes allow it to accept 87.3kb/s in committed-to data though this is not directly available within
state, so we exclude it from this illustration, though including it with the input data would change the results little.
## 14
This is detailed athttps://hackmd.io/@XXX9CM1uSSCWVNFRYaSB5g/HJarTUhJAand intended to be updated as we get more information.
## 15
It is conservative since we don’t take into account that the source code was originally compiled intoevmcode and thus thepvm
machine code will replicate architectural artifacts and thus is very likely to be pessimistic. As an example, all arithmetic operations inevm
are 256-bit and 64-bit nativepvmis being forced to honor this even if the source code only actually required 64-bit values.
## 16
We speculate that the substantial range could possibly be caused in part by the major architectural differences between theevm isa
and typical modern hardware.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202534
Benchmarking and regression tests show that the pro-
totypepvmengine has a fixed preprocessing overhead of
around 5ns/byte of program code and, for arithmetic-
heavy tasks at least, a marginal factor of 1.6-2% com-
pared toevmexecution, implying an asymptotic speedup
of around 50-60x. For machine code 1mbin size expected
to take of the order of a second to compute, the com-
pilation cost becomes only 0.5% of the overall time.
## 17
For code not inherently suited to the 256-bitevm isa,
we would expect substantially improved relative execu-
tion times onpvm, though more work must be done in
order to gain confidence that these speed-ups are broadly
applicable.
If we allow for preprocessing to take up to the same
component within execution as the marginal cost (owing
to, for example, an extremely large but short-running pro-
gram) and for thepvmmetering to imply a safety overhead
of 2x to execution speeds, then we can expect a
## J
amcore
to be able to process the equivalent of around 1,500evm
gas/μs. Owing to the crudeness of our analysis we might
reasonably predict it to be somewhere within a factor of
three either way—i.e. 500-5,000evmgas/μs.
## J
amcores are each capable of 2mb/s bandwidth, which
must include any statei/oand data which must be newly
introduced (e.g. transactions). While writes come at com-
paratively little cost to the core, only requiring hashing to
determine an eventual updated Merkle root, reads must
be witnessed, with each one costing around 640 bytes of
witness conservatively assuming a one-million entry bi-
nary Merkle trie. This would result in a maximum of a
little over 3k reads/second/core, with the exact amount
dependent upon how much of the bandwidth is used for
newly introduced input data.
Aggregating everything across
## J
am, excepting accu-
mulation which could add further throughput, numbers
can be multiplied by 341 (with the caveat that each one’s
computation cannot interfere with any of the others’ ex-
cept through state oraclization and accumulation). Unlike
forroll-up chaindesigns such as Polkadot and Ethereum,
there is no need to have persistently fragmented state.
Smart-contract state may be held in a coherent format on
the
## J
amchain so long as any updates are made through
the 8kb/core/sec work-results, which would need to con-
tain only the hashes of the altered contracts’ state roots.
Under our modelling assumptions, we can therefore
summarize:
## Eth. L1
## J
amCore
## J
am
## Compute (evmgas/μs)1.25
## †
## 500-5,000  0.15-1.5m
State writes (s
## −1
## )57
## †
n/an/a
State reads (s
## −1
## )595
## †
## 4k
## ‡
## 1.4m
## ‡
Input data (s
## −1
## )78kb
## †
## 2mb
## ‡
## 682mb
## ‡
What we can see is that
## J
am’s overall predicted per-
formance profile implies it could be comparable to many
thousands of that of the basic Ethereum L1 chain. The
large factor here is essentially due to three things: spacial
parallelism, as
## J
amcan host several hundred cores under
its security apparatus; temporal parallelism, as
## J
amtar-
gets continuous execution for its cores and pipelines much
of the computation between blocks to ensure a constant,
optimal workload; and platform optimization by using a
vmand gas model which closely fits modern hardware ar-
chitectures.
It must however be understood that this is a provi-
sional and crude estimation only. It is included only for
the purpose of expressing
## J
am’s performance in tangible
terms. Specifically, it does not take into account:
●that these numbers are based on real performance
of Ethereum and performance modelling of
## J
am
(though our models are based on real-world per-
formance of the components);
●any L2 scaling which may be possible with either
## J
amor Ethereum;
●the state partitioning which uses of
## J
amwould
imply;
●the as-yet unfixed gas model for thepvm;
●thatpvm/evmcomparisons are necessarily impre-
cise;
## ●(
## †
) all figures for Ethereum L1 are drawn from
the same resource: on average each figure will be
only
## 1
~4of this maximum.
## ●(
## ‡
) the state reads and input data figures for
## J
am
are drawn from the same resource: on average
each figure will be only
## 1
~2of this maximum.
We leave it as further work for an empirical analysis of
performance and an analysis and comparison between
## J
am
and the aggregate of a hypothetical Ethereum ecosystem
which included some maximal amount of L2 deployments
together with full Dank-sharding and any other additional
consensus elements which they would require. This, how-
ever, is out of scope for the present work.
21.Conclusion
We have introduced a novel computation model which
is able to make use of pre-existing crypto-economic mech-
anisms in order to deliver major improvements in scala-
bility without causing persistent state-fragmentation and
thus sacrificing overall cohesion. We call this overall pat-
tern collect-refine-join-accumulate. Furthermore, we have
formally defined the on-chain portion of this logic, essen-
tially the join-accumulate portion. We call this protocol
the
## J
amchain.
We argue that the model of
## J
amprovides a novel “sweet
spot”, allowing for massive amounts of computation to
be done in secure, resilient consensus compared to fully-
synchronous models, and yet still have strict guarantees
about both timing and integration of the computation
into some singleton state machine unlike persistently frag-
mented models.
21.1.Further Work.While we are able to estimate the-
oretical computation possible given some basic assump-
tions and even make broad comparisons to existing sys-
tems, practical numbers are invaluable. We believe the
model warrants further empirical research in order to bet-
ter understand how these theoretical limits translate into
real-world performance. We feel a proper cost analysis
and comparison to pre-existing protocols would also be an
excellent topic for further work.
We can be reasonably confident that the design of
## J
am
allows it to host a service under which Polkadotparachains
## 17
As an example, our odd-product benchmark, a very much pure-compute arithmetic task, execution takes 58s onevm, and 1.04s within
ourpvmprototype, including all preprocessing.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202535
could be validated, however further prototyping work is
needed to understand the possible throughput which a
pvm-powered metering system could support. We leave
such a report as further work. Likewise, we have also
intentionally omitted details of higher-level protocol ele-
ments including cryptocurrency, coretime sales, staking
and regular smart-contract functionality.
A number of potential alterations to the protocol de-
scribed here are being considered in order to make prac-
tical utilization of the protocol easier. These include:
●Synchronous calls between services in accumulate.
●Restrictions on thetransferfunction in order to
allow for substantial parallelism over accumula-
tion.
●The possibility of reserving substantial additional
computation capacity during accumulate under
certain conditions.
●Introducing Merklization into the Work Package
format in order to obviate the need to have the
whole package downloaded in order to evaluate
its authorization.
The networking protocol is also left intentionally un-
defined at this stage and its description must be done in
a follow-up proposal.
Validator performance is not presently tracked on-
chain. We do expect this to be tracked on-chain in the
final revision of the
## J
amprotocol, but its specific format
is not yet certain and it is therefore omitted at present.
22.Acknowledgements
Much of this present work is based in large part on the
work of others. The Web3 Foundation research team and
in particular Alistair Stewart and Jeff Burdges are respon-
sible forElves, the security apparatus of Polkadot which
enables the possibility of in-core computation for
## J
am.
The same team is responsible for Sassafras,Grandpaand
## Beefy.
Safrole is a mild simplification of Sassafras and was
made under the careful review of Davide Galassi and Al-
istair Stewart.
The original CoreJamrfcwas refined under the re-
view of Bastian Köcher and Robert Habermeier and most
of the key elements of that proposal have made their way
into the present work.
Thepvmis a formalization of a partially simplified
PolkaVMsoftware prototype, developed by Jan Bujak.
Cyrill Leutwiler contributed to the empirical analysis of
thepvmreported in the present work.
ThePolkaJamteam and in particular Arkadiy
Paronyan, Emeric Chevalier and Dave Emett have been
instrumental in the design of the lower-level aspects of
the
## J
amprotocol, especially concerning Merklization and
i/o.
Numerous contributors to the repository since publica-
tion have helped correct errors. Thank you to all.
And, of course, thanks to the awesome Lemon Jelly,
a.k.a. Fred Deakin and Nick Franglen, for three of the
most beautiful albums ever produced, the cover art of the
first of which was inspiration for this paper’s background
art.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202536
AppendixA.Polkadot Virtual Machine
A.1.Basic Definition.We declare the generalpvmfunctionΨ. We assume a single-step invocation function defineΨ
## 1
and define the fullpvmrecursively as a sequence of such mutations up until the single-step mutation results in a halting
condition. We additionally define the function deblob which extracts the instruction data, opcode bitmask and dynamic
jump table from a program blob:
## Ψ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## B,N
## R
## ,N
## G
## ,⟦N
## R
## ⟧
## 13
## ,M
## ⎫
## ⎭
## →
## ⎧
## ⎪
## ⎩
## 
## ∎,☇,∞
## 
## ∪{
## F
## ,
## ̵
h}×N
## R
## ,N
## R
## ,Z
## G
## ,⟦N
## R
## ⟧
## 13
## ,M
## ⎫
## ⎪
## ⎭
## (p,ı,ρ,φ,μ)↦
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## Ψ(p,ı
## ′
## ,ρ
## ′
## ,φ
## ′
## ,μ
## ′
## )ifε=▸
## (∞,ı,ρ
## ′
## ,φ,μ)ifρ
## ′
## <0
## (ε,0,ρ
## ′
## ,φ
## ′
## ,μ
## ′
## )ifε∈
## 
## ☇,∎
## 
## (ε,ı,ρ
## ′
## ,φ,μ)otherwise
where
## 
ε,ı
## ′
## ,ρ
## ′
## ,φ
## ′
## ,μ
## ′
## 
## =
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## Ψ
## 1
## (c,k,j,ı,ρ,φ,μ)if(c,k,j)=deblob(p)
## 
## ☇,ı,ρ,φ,μ
## 
otherwise
## (A.1)
deblob∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## B→
## ⎧
## ⎩
B,b,⟦N
## R
## ⟧
## ⎫
## ⎭
## ∪ ∇
p↦
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
(c,k,j)if∃!c,k,j∶p=E(SjS)⌢E
## 1
(z)⌢E(ScS)⌢E
z
(j)⌢E(c)⌢E(k),SkS=ScS
## ∇otherwise
## (A.2)
Thepvmexit reasonε∈
## 
## ∎,☇,∞
## 
## ∪{
## F
## ,
## ̵
h}×N
## R
may be one of regular halt∎, panic☇or out-of-gas∞, or alternatively a
host-call
## ̵
h, in which the host-call identifier is associated, or page-fault
## F
in which case the address intoramis associated.
Assuming the program blob is valid (which can be validated statically), some gas is always charged whenever execution
is attempted. This is the case even if no instruction is effectively executed and machine state is unchanged (i.e. the result
state is equal to the parameter).
In the case of a final halt, either through panic or success, the instruction counter returned is zero. In all other
cases, the return value of the instruction counter indexes the onewhich caused the exit to happenand the machine state
represents the prior state of said instruction, thus ensuringde factoconsistency. In order to continue beyond these
exit cases, some environmental factor must be adjusted; for a page-fault,rammust be changed, for a gas-underflow,
more gas must be supplied and for a host-call, the instruction-counter must be incremented and the relevant host-call
state-transition performed.
A.2.Instructions, Opcodes and Skip-distance.The program blobpis split into a series of octets which make
up theinstruction datacand theopcode bitmaskkas well as thedynamic jump table,j. The former two imply an
instruction sequence, and by extension abasic-block sequence, itself a sequence of indices of the instructions which follow
ablock-terminationinstruction.
The latter, dynamic jump table, is a sequence of indices into the instruction data blob and is indexed into when
dynamically-computed jumps are taken. It is encoded as a sequence of natural numbers (i.e. non-negative integers) each
encoded with the same length in octets. This length, termzabove, is itself encoded prior.
Thepvmcounts instructions in octet terms (rather than in terms of instructions) and it is thus necessary to define
which octets represent the beginning of an instruction, i.e. the opcode octet, and which do not. This is the purpose ofk,
the instruction-opcode bitmask. We assert that the length of the bitmask is equal to the length of the instruction blob.
We define the Skip function skip which provides the number of octets, minus one, to the next instruction’s opcode,
given the index of instruction’s opcode index intoc(and by extensionk):
## (A.3)
skip∶
## N→N
i↦min(24, j∈N∶(k⌢[1,1,...])
i+1+j
## =1)
The Skip function appendskwith a sequence of set bits in order to ensure a well-defined result for the final instruction
skip(ScS−1).
Given some instruction-indexi, its opcode is readily expressed asc
i
and the distance in octets to move forward to the
next instruction is1+skip(i). However, each instruction’s “length” (defined as the number of contiguous octets starting
with the opcode which are needed to fully define the instruction’s semantics) is left implicit though limited to being at
most 16.
We defineζas being equivalent to the instructionscexcept with an indefinite sequence of zeroes suﬀixed to ensure that
no out-of-bounds access is possible. This effectively defines any otherwise-undefined arguments to the final instruction
and ensures that a trap will occur if the program counter passes beyond the program code. Formally:
## (A.4)
ζ≡c⌢[0,0,...]
A.3.Basic Blocks and Termination Instructions.Instructions of the following opcodes are considered basic-block
termination instructions; other thantrap&fallthrough, they correspond to instructions which may define the instruction-
counter to be something other than its prior value plus the instruction’s skip amount:
●Trap and fallthrough:trap,fallthrough
●Jumps:
jump
## ,
jump_ind
●Load-and-Jumps:load_imm_jump,load_imm_jump_ind

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202537
●Branches:branch_eq,branch_ne,branch_ge_u,branch_ge_s,branch_lt_u,branch_lt_s,branch_eq_imm,
branch_ne_imm
●Immediate branches:branch_lt_u_imm,branch_lt_s_imm,branch_le_u_imm,branch_le_s_imm,branch_ge_u_imm,
branch_ge_s_imm,branch_gt_u_imm,branch_gt_s_imm
We denote this set, as opcode indices rather than names, asT, which is a subset of all valid opcode indicesU. We
define the instruction opcode indices denoting the beginning of basic-blocks asπ:
(A.5)π≡
## 
## {0}∪
## 
n+1+skip(n)
## T
n∈N
ScS
## ∧k
n
## =1∧c
n
## ∈T
## 
∩{nSk
n
## =1∧c
n
## ∈U}
A.4.Single-Step State Transition.We must now define the single-steppvmstate-transition functionΨ
## 1
## :
## (A.6)Ψ
## 1
## ∶
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
B,b,⟦N
## R
## ⟧,N
## R
## ,N
## G
## ,⟦N
## R
## ⟧
## 13
## ,M
## ⎫
## ⎭
## →
## ⎧
## ⎪
## ⎩
## 
## ☇,∎,▸
## 
## ∪{
## F
## ,
## ̵
h}×N
## R
## ,N
## R
## ,Z
## G
## ,⟦N
## R
## ⟧
## 13
## ,M
## ⎫
## ⎪
## ⎭
## (c,k,j,ı,ρ,φ,μ)↦
## 
ε
## ∗
## ,ı
## ∗
## ,ρ
## ∗
## ,φ
## ∗
## ,μ
## ∗
## 
During the course of executing instructionsrammay be accessed. When an index oframbelow2
## 16
is required, the
machine always panics immediately without further changes to its state regardless of the apparent (in)accessibility of
the value. Otherwise, should the given index oframnot be accessible then machine state remains unchanged and the
exit reason is a fault with the lowest inaccessiblepage addressto be read. Similarly, whererammust be mutated and
yet mutable access is not possible, then machine state is unchanged, and the exit reason is a fault with the lowest page
address to be written which is inaccessible.
Formally, letrandwbe the set of indices by whichμmust be subscripted for inspection and mutation respectively
in order to calculate the result ofΨ
## 1
. We define the memory-access exceptional execution stateε
μ
which shall, if not▸,
singly effect the returned return ofΨ
## 1
as following:
letx=
## 
x
## T
x∈r∧xmod 2
## 32
## ~∈V
μ
## ∨x∈w∧xmod 2
## 32
## ~∈V
## ∗
μ
## 
## (A.7)
## 
ε
## ∗
## ,ı
## ∗
## ,ρ
## ∗
## ,φ
## ∗
## ,μ
## ∗
## 
## =
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## (ε,ı
## ′
## ,ρ
## ′
## ,φ
## ′
## ,μ
## ′
## )ifx={}
## 
## ☇,ı,ρ,φ,μ
## 
ifmin(x)mod 2
## 32
## <2
## 16
## 
## F
## ×Z
## P

min(x)mod 2
## 32
## ÷Z
## P
## 
## ,ı,ρ,φ,μ
## 
otherwise
## (A.8)
We defineεtogether with the posterior values of regular execution (denoted as prime) of each of the items of the
machine state as being in accordance with the table below. When transitioning machine state for an instruction, a
number of conditions typically hold true and instructions are defined essentially by their exceptions to these rules.
Specifically, the machine does not halt, the instruction counter increments by one, the gas remaining is reduced by the
amount corresponding to the instruction type andram& registers are unchanged. Formally:
(A.9)ε=▸,  ı
## ′
=ı+1+skip(ı),  ρ
## ′
## =ρ−ρ
## ∆
,  φ
## ′
=φ,  μ
## ′
=μexcept as indicated
In the case thatΨ
## 1
takes theε
μ
We define signed/unsigned transitions for various octet widths:
## Z
n∈N
## ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## N
## 2
## 8n
## →Z
## −2
## 8n−1
## ...2
## 8n−1
a↦
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
aifa<2
## 8n−1
a−2
## 8n
otherwise
## (A.10)
## Z
## −1
n∈N
## ∶
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## Z
## −2
## 8n−1
## ...2
## 8n−1
## →N
## 2
## 8n
a↦(2
## 8n
## +a)mod 2
## 8n
## (A.11)
## B
n∈N
## ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎩
## N
## 2
## 8n
## →b
## 8n
x↦y∶ ∀i∈N
## 8n
## ∶y[i]⇔
x
## 2
i
## mod 2
## (A.12)
## B
## −1
n∈N
## ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎩
b
## 8n
## →N
## 2
## 8n
x↦y∶
## ∑
i∈N
## 8n
x
i
## ⋅2
i
## (A.13)
## ←Ð
## B
n∈N
## ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎩
## N
## 2
## 8n
## →b
## 8n
x↦y∶ ∀i∈N
## 8n
## ∶y[8n−1−i]⇔
x
## 2
i
## mod 2
## (A.14)
## ←Ð
## B
## −1
n∈N
## ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎩
b
## 8n
## →N
## 2
## 8n
x↦y∶
## ∑
i∈N
## 8n
x
## 8n−1−i
## ⋅2
i
## (A.15)
Immediate arguments are encoded in little-endian format with the most-significant bit being the sign bit. They may
be compactly encoded by eliding more significant octets. Elided octets are assumed to be zero if themsbof the value is
zero, and 255 otherwise. This allows for compact representation of both positive and negative encoded values. We thus
define the signed extension function operating on an input ofnoctets asX
n
## :
## X
n∈{0,1,2,3,4,8}
## ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎩
## N
## 2
## 8n
## →N
## R
x↦x+
x
## 2
## 8n−1
## (2
## 64
## −2
## 8n
## )
## (A.16)

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202538
Any alterations of the program counter stemming from a static jump, call or branch must be to the start of a basic
block or else a panic occurs. Hypotheticals are not considered. Formally:
(A.17)branch(b,C)Ô⇒
## 
ε,ı
## ′
## 
## =
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
(▸,ı)if¬C
## 
## ☇,ı
## 
otherwise ifb~∈π
## (▸,b)otherwise
Jumps whose next instruction is dynamically computed must use an address which may be indexed into the jump-
tablej. Through a quirk of tooling
## 18
, we define the dynamic address required by the instructions as the jump table index
incremented by one and then multiplied by our jump alignment factorZ
## A
## =2.
As with other irregular alterations to the program counter, target code index must be the start of a basic block or
else a panic occurs. Formally:
## (A.18)
djump(a)Ô⇒
## 
ε,ı
## ′
## 
## =
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## (∎,ı)ifa=2
## 32
## −2
## 16
## 
## ☇,ı
## 
otherwise ifa=0∨a>SjS⋅Z
## A
∨amodZ
## A
## ≠0∨j
## (
a
## ~Z
## A
## )−1
## ~∈π
## (▸,j
## (
a
## ~Z
## A
## )−1
## )otherwise
A.5.Instruction Tables.Only instructions which are defined in the following tables and whose opcode has its corre-
sponding bit set in the bitmask are considered valid, otherwise the instruction behaves as-if its opcode was equal to zero.
AssumingUdenotes all valid opcode indices, formally:
(A.19)opcode∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## N→N
n↦
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
c
n
ifk
n
## =1∧c
n
## ∈U
## 0otherwise
We assume the skip lengthℓis well-defined:
## (A.20)
## ℓ≡skip(ı)
A.5.1.Instructions without Arguments.
ζ
ı
## Nameρ
## ∆
## Mutations
## 0trap1ε=☇
## 1fallthrough1
A.5.2.Instructions with Arguments of One Immediate.
## (A.21)
letl
## X
=min(4,ℓ),  ν
## X
## ≡X
l
## X
## 
## E
## −1
l
## X
## (ζ
ı+1⋅⋅⋅+l
## X
## )
## 
ζ
ı
## Nameρ
## ∆
## Mutations
## 10ecalli1ε=
## ̵
h×ν
## X
A.5.3.Instructions with Arguments of One Register and One Extended Width Immediate.
## (A.22)
letr
## A
## =min(12,ζ
ı+1
mod 16),  φ
## ′
## A
## ≡φ
## ′
r
## A
,  ν
## X
## ≡E
## −1
## 8
## (ζ
ı+2⋅⋅⋅+8
## )
ζ
ı
## Nameρ
## ∆
## Mutations
## 20load_imm_641φ
## ′
## A
## =ν
## X
## 18
The popular code generation backendllvmrequires and assumes in its code generation that dynamically computed jump destinations
always have a certain memory alignment. Since at present we depend on this for our tooling, we must acquiesce to its assumptions.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202539
ζ
ı
## Nameρ
## ∆
## Mutations
A.5.4.Instructions with Arguments of Two Immediates.
## (A.23)
letl
## X
## =min(4,ζ
ı+1
mod 8),ν
## X
## ≡X
l
## X
## 
## E
## −1
l
## X
## (ζ
ı+2⋅⋅⋅+l
## X
## )
## 
letl
## Y
## =min(4,max(0,ℓ−l
## X
−1)),   ν
## Y
## ≡X
l
## Y
## 
## E
## −1
l
## Y
## (ζ
ı+2+l
## X
## ⋅⋅⋅+l
## Y
## )
## 
ζ
ı
## Nameρ
## ∆
## Mutations
## 30store_imm_u81μ
## ′
## ↺
ν
## X
## =ν
## Y
mod 2
## 8
## 31store_imm_u161μ
## ′
## ↺
ν
## X
## ⋅⋅⋅+2
## =E
## 2
## 
ν
## Y
mod 2
## 16
## 
## 32store_imm_u321μ
## ′
## ↺
ν
## X
## ⋅⋅⋅+4
## =E
## 4
## 
ν
## Y
mod 2
## 32
## 
## 33store_imm_u641μ
## ′
## ↺
ν
## X
## ⋅⋅⋅+8
## =E
## 8
## (ν
## Y
## )
A.5.5.Instructions with Arguments of One Offset.
## (A.24)
letl
## X
=min(4,ℓ),  ν
## X
≡ı+Z
l
## X
## (E
## −1
l
## X
## (ζ
ı+1⋅⋅⋅+l
## X
## ))
ζ
ı
## Nameρ
## ∆
## Mutations
## 40jump1branch(ν
## X
## ,⊺)
A.5.6.Instructions with Arguments of One Register & One Immediate.
## (A.25)
letr
## A
## =min(12,ζ
ı+1
mod 16),   φ
## A
## ≡φ
r
## A
,  φ
## ′
## A
## ≡φ
## ′
r
## A
letl
## X
=min(4,max(0,ℓ−1)),   ν
## X
## ≡X
l
## X
## 
## E
## −1
l
## X
## (ζ
ı+2⋅⋅⋅+l
## X
## )
## 
ζ
ı
## Nameρ
## ∆
## Mutations
## 50jump_ind1djump((φ
## A
## +ν
## X
## )mod 2
## 32
## )
## 51load_imm1φ
## ′
## A
## =ν
## X
## 52load_u81φ
## ′
## A
## =μ
## ↺
ν
## X
## 53load_i81φ
## ′
## A
## =X
## 1
## μ
## ↺
ν
## X
## 
## 54load_u161φ
## ′
## A
## =E
## −1
## 2
## μ
## ↺
ν
## X
## ⋅⋅⋅+2
## 
## 55load_i161φ
## ′
## A
## =X
## 2
## E
## −1
## 2
## μ
## ↺
ν
## X
## ⋅⋅⋅+2
## 
## 56load_u321φ
## ′
## A
## =E
## −1
## 4
## μ
## ↺
ν
## X
## ⋅⋅⋅+4
## 
## 57load_i321φ
## ′
## A
## =X
## 4
## E
## −1
## 4
## μ
## ↺
ν
## X
## ⋅⋅⋅+4
## 
## 58load_u641φ
## ′
## A
## =E
## −1
## 8
## μ
## ↺
ν
## X
## ⋅⋅⋅+8
## 
## 59store_u81μ
## ′
## ↺
ν
## X
## =φ
## A
mod 2
## 8
## 60store_u161μ
## ′
## ↺
ν
## X
## ⋅⋅⋅+2
## =E
## 2
## 
φ
## A
mod 2
## 16
## 
## 61store_u321μ
## ′
## ↺
ν
## X
## ⋅⋅⋅+4
## =E
## 4
## 
φ
## A
mod 2
## 32
## 
## 62store_u641μ
## ′
## ↺
ν
## X
## ⋅⋅⋅+8
## =E
## 8
## (φ
## A
## )
A.5.7.Instructions with Arguments of One Register & Two Immediates.
## (A.26)
letr
## A
## =min(12,ζ
ı+1
mod 16),φ
## A
## ≡φ
r
## A
,  φ
## ′
## A
## ≡φ
## ′
r
## A
letl
## X
## =min(4,
ζ
ı+1
## 16
## mod 8),ν
## X
## =X
l
## X
## 
## E
## −1
l
## X
## (ζ
ı+2⋅⋅⋅+l
## X
## )
## 
letl
## Y
## =min(4,max(0,ℓ−l
## X
−1)),   ν
## Y
## =X
l
## Y
## 
## E
## −1
l
## Y
## (ζ
ı+2+l
## X
## ⋅⋅⋅+l
## Y
## )
## 

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202540
ζ
ı
## Nameρ
## ∆
## Mutations
## 70store_imm_ind_u81μ
## ′
## ↺
φ
## A
## +ν
## X
## =ν
## Y
mod 2
## 8
## 71store_imm_ind_u161μ
## ′
## ↺
φ
## A
## +ν
## X
## ⋅⋅⋅+2
## =E
## 2
## 
ν
## Y
mod 2
## 16
## 
## 72store_imm_ind_u321μ
## ′
## ↺
φ
## A
## +ν
## X
## ⋅⋅⋅+4
## =E
## 4
## 
ν
## Y
mod 2
## 32
## 
## 73store_imm_ind_u641μ
## ′
## ↺
φ
## A
## +ν
## X
## ⋅⋅⋅+8
## =E
## 8
## (ν
## Y
## )
A.5.8.Instructions with Arguments of One Register, One Immediate and One Offset.
## (A.27)
letr
## A
## =min(12,ζ
ı+1
mod 16),φ
## A
## ≡φ
r
## A
,  φ
## ′
## A
## ≡φ
## ′
r
## A
letl
## X
## =min(4,
ζ
ı+1
## 16
## mod 8),ν
## X
## =X
l
## X
## 
## E
## −1
l
## X
## (ζ
ı+2⋅⋅⋅+l
## X
## )
## 
letl
## Y
## =min(4,max(0,ℓ−l
## X
−1)),   ν
## Y
=ı+Z
l
## Y
## (E
## −1
l
## Y
## (ζ
ı+2+l
## X
## ⋅⋅⋅+l
## Y
## ))
ζ
ı
## Nameρ
## ∆
## Mutations
## 80load_imm_jump1branch(ν
## Y
,⊺),   φ
## ′
## A
## =ν
## X
## 81branch_eq_imm1branch(ν
## Y
## ,φ
## A
## =ν
## X
## )
## 82branch_ne_imm1branch(ν
## Y
## ,φ
## A
## ≠ν
## X
## )
## 83branch_lt_u_imm1branch(ν
## Y
## ,φ
## A
## <ν
## X
## )
## 84branch_le_u_imm1branch(ν
## Y
## ,φ
## A
## ≤ν
## X
## )
## 85branch_ge_u_imm1branch(ν
## Y
## ,φ
## A
## ≥ν
## X
## )
## 86branch_gt_u_imm1branch(ν
## Y
## ,φ
## A
## >ν
## X
## )
## 87branch_lt_s_imm1branch(ν
## Y
## ,Z
## 8
## (φ
## A
## )<Z
## 8
## (ν
## X
## ))
## 88branch_le_s_imm1branch(ν
## Y
## ,Z
## 8
## (φ
## A
## )≤Z
## 8
## (ν
## X
## ))
## 89branch_ge_s_imm1branch(ν
## Y
## ,Z
## 8
## (φ
## A
## )≥Z
## 8
## (ν
## X
## ))
## 90branch_gt_s_imm1branch(ν
## Y
## ,Z
## 8
## (φ
## A
## )>Z
## 8
## (ν
## X
## ))
A.5.9.Instructions with Arguments of Two Registers.
## (A.28)
letr
## D
## =min(12,(ζ
ı+1
)mod 16),   φ
## D
## ≡φ
r
## D
,  φ
## ′
## D
## ≡φ
## ′
r
## D
letr
## A
## =min(12,
ζ
ı+1
## 16
## ),φ
## A
## ≡φ
r
## A
,  φ
## ′
## A
## ≡φ
## ′
r
## A
ζ
ı
## Nameρ
## ∆
## Mutations
## 100move_reg1φ
## ′
## D
## =φ
## A
## 101sbrk1
φ
## ′
## D
≡min(x∈N
## R
## )∶
x≥h
## N
x⋅⋅⋅+φ
## A
## ~⊆V
μ
## N
x⋅⋅⋅+φ
## A
## ⊆V
## ∗
μ
## ′
## 102count_set_bits_641φ
## ′
## D
## =
## 63
## ∑
i=0
## B
## 8
## (φ
## A
## )
i
## 103count_set_bits_321φ
## ′
## D
## =
## 31
## ∑
i=0
## B
## 4
## (φ
## A
mod 2
## 32
## )
i
## 104leading_zero_bits_641φ
## ′
## D
=max(n∈N
## 65
## )where
i<n
## ∑
i=0
## ←Ð
## B
## 8
## (φ
## A
## )
i
## =0
## 105leading_zero_bits_321φ
## ′
## D
=max(n∈N
## 33
## )where
i<n
## ∑
i=0
## ←Ð
## B
## 4
## (φ
## A
mod 2
## 32
## )
i
## =0
## 106trailing_zero_bits_641φ
## ′
## D
=max(n∈N
## 65
## )where
i<n
## ∑
i=0
## B
## 8
## (φ
## A
## )
i
## =0
## 107trailing_zero_bits_321φ
## ′
## D
=max(n∈N
## 33
## )where
i<n
## ∑
i=0
## B
## 4
## (φ
## A
mod 2
## 32
## )
i
## =0

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202541
ζ
ı
## Nameρ
## ∆
## Mutations
## 108sign_extend_81φ
## ′
## D
## =Z
## −1
## 8
## (Z
## 1
## (φ
## A
mod 2
## 8
## ))
## 109sign_extend_161φ
## ′
## D
## =Z
## −1
## 8
## (Z
## 2
## (φ
## A
mod 2
## 16
## ))
## 110zero_extend_161φ
## ′
## D
## =φ
## A
mod 2
## 16
111reverse_bytes1∀i∈N
## 8
## ∶E
## 8
## (φ
## ′
## D
## )
i
## =E
## 8
## (φ
## A
## )
## 7−i
Note, the termhabove refers to the beginning of the heap, the second major section of memory as defined in equation
A.42as2Z
## Z
+Z(SoS). Ifsbrkinstruction is invoked on apvminstance which does not have such a memory layout, then
h=0.
A.5.10.Instructions with Arguments of Two Registers & One Immediate.
## (A.29)
letr
## A
## =min(12,(ζ
ı+1
)mod 16),   φ
## A
## ≡φ
r
## A
,  φ
## ′
## A
## ≡φ
## ′
r
## A
letr
## B
## =min(12,
ζ
ı+1
## 16
## ),φ
## B
## ≡φ
r
## B
,  φ
## ′
## B
## ≡φ
## ′
r
## B
letl
## X
## =min(4,max(0,ℓ−1)),ν
## X
## ≡X
l
## X
## 
## E
## −1
l
## X
## (ζ
ı+2⋅⋅⋅+l
## X
## )
## 
ζ
ı
## Nameρ
## ∆
## Mutations
## 120store_ind_u81μ
## ′
## ↺
φ
## B
## +ν
## X
## =φ
## A
mod 2
## 8
## 121store_ind_u161μ
## ′
## ↺
φ
## B
## +ν
## X
## ⋅⋅⋅+2
## =E
## 2
## 
φ
## A
mod 2
## 16
## 
## 122store_ind_u321μ
## ′
## ↺
φ
## B
## +ν
## X
## ⋅⋅⋅+4
## =E
## 4
## 
φ
## A
mod 2
## 32
## 
## 123store_ind_u641μ
## ′
## ↺
φ
## B
## +ν
## X
## ⋅⋅⋅+8
## =E
## 8
## (φ
## A
## )
## 124load_ind_u81φ
## ′
## A
## =μ
## ↺
φ
## B
## +ν
## X
## 125load_ind_i81φ
## ′
## A
## =Z
## −1
## 8
## (Z
## 1
## (μ
## ↺
φ
## B
## +ν
## X
## ))
## 126load_ind_u161φ
## ′
## A
## =E
## −1
## 2
## μ
## ↺
φ
## B
## +ν
## X
## ⋅⋅⋅+2
## 
## 127load_ind_i161φ
## ′
## A
## =Z
## −1
## 8
## (Z
## 2
## (E
## −1
## 2
## μ
## ↺
φ
## B
## +ν
## X
## ⋅⋅⋅+2
## ))
## 128load_ind_u321φ
## ′
## A
## =E
## −1
## 4
## μ
## ↺
φ
## B
## +ν
## X
## ⋅⋅⋅+4
## 
## 129load_ind_i321φ
## ′
## A
## =Z
## −1
## 8
## (Z
## 4
## (E
## −1
## 4
## μ
## ↺
φ
## B
## +ν
## X
## ⋅⋅⋅+4
## ))
## 130load_ind_u641φ
## ′
## A
## =E
## −1
## 8
## μ
## ↺
φ
## B
## +ν
## X
## ⋅⋅⋅+8
## 
## 131add_imm_321φ
## ′
## A
## =X
## 4
## 
## (φ
## B
## +ν
## X
## )mod 2
## 32
## 
132and_imm1∀i∈N
## 64
## ∶B
## 8
## (φ
## ′
## A
## )
i
## =B
## 8
## (φ
## B
## )
i
## ∧B
## 8
## (ν
## X
## )
i
133xor_imm1∀i∈N
## 64
## ∶B
## 8
## (φ
## ′
## A
## )
i
## =B
## 8
## (φ
## B
## )
i
## ⊕B
## 8
## (ν
## X
## )
i
134or_imm1∀i∈N
## 64
## ∶B
## 8
## (φ
## ′
## A
## )
i
## =B
## 8
## (φ
## B
## )
i
## ∨B
## 8
## (ν
## X
## )
i
## 135mul_imm_321φ
## ′
## A
## =X
## 4
## 
## (φ
## B
## ⋅ν
## X
## )mod 2
## 32
## 
## 136set_lt_u_imm1φ
## ′
## A
## =φ
## B
## <ν
## X
## 137set_lt_s_imm1φ
## ′
## A
## =Z
## 8
## (φ
## B
## )<Z
## 8
## (ν
## X
## )
## 138shlo_l_imm_321φ
## ′
## A
## =X
## 4
## 
## (φ
## B
## ⋅2
ν
## X
mod 32
## )mod 2
## 32
## 
## 139shlo_r_imm_321φ
## ′
## A
## =X
## 4
## 
φ
## B
mod 2
## 32
## ÷2
ν
## X
mod 32
## 
## 140shar_r_imm_321φ
## ′
## A
## =Z
## −1
## 8
## (

## Z
## 4
## (φ
## B
mod 2
## 32
## )÷2
ν
## X
mod 32
## 
## )
## 141neg_add_imm_321φ
## ′
## A
## =X
## 4
## 
## (ν
## X
## +2
## 32
## −φ
## B
## )mod 2
## 32
## 
## 142set_gt_u_imm1φ
## ′
## A
## =φ
## B
## >ν
## X
## 143set_gt_s_imm1φ
## ′
## A
## =Z
## 8
## (φ
## B
## )>Z
## 8
## (ν
## X
## )
## 144shlo_l_imm_alt_321φ
## ′
## A
## =X
## 4
## 
## (ν
## X
## ⋅2
φ
## B
mod 32
## )mod 2
## 32
## 
## 145shlo_r_imm_alt_321φ
## ′
## A
## =X
## 4
## 
ν
## X
mod 2
## 32
## ÷2
φ
## B
mod 32
## 
## 146shar_r_imm_alt_321φ
## ′
## A
## =Z
## −1
## 8
## (

## Z
## 4
## (ν
## X
mod 2
## 32
## )÷2
φ
## B
mod 32
## 
## )

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202542
ζ
ı
## Nameρ
## ∆
## Mutations
## 147cmov_iz_imm1φ
## ′
## A
## =
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
ν
## X
ifφ
## B
## =0
φ
## A
otherwise
## 148cmov_nz_imm1φ
## ′
## A
## =
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
ν
## X
ifφ
## B
## ≠0
φ
## A
otherwise
## 149add_imm_641φ
## ′
## A
## =(φ
## B
## +ν
## X
## )mod 2
## 64
## 150mul_imm_641φ
## ′
## A
## =(φ
## B
## ⋅ν
## X
## )mod 2
## 64
## 151shlo_l_imm_641φ
## ′
## A
## =X
## 8
## 
## (φ
## B
## ⋅2
ν
## X
mod 64
## )mod 2
## 64
## 
## 152shlo_r_imm_641φ
## ′
## A
## =X
## 8
## 
φ
## B
## ÷2
ν
## X
mod 64
## 
## 153shar_r_imm_641φ
## ′
## A
## =Z
## −1
## 8
## (

## Z
## 8
## (φ
## B
## )÷2
ν
## X
mod 64
## 
## )
## 154neg_add_imm_641φ
## ′
## A
## =(ν
## X
## +2
## 64
## −φ
## B
## )mod 2
## 64
## 155shlo_l_imm_alt_641φ
## ′
## A
## =(ν
## X
## ⋅2
φ
## B
mod 64
## )mod 2
## 64
## 156shlo_r_imm_alt_641φ
## ′
## A
## =

ν
## X
## ÷2
φ
## B
mod 64
## 
## 157shar_r_imm_alt_641φ
## ′
## A
## =Z
## −
## 1
## 8
## (

## Z
## 8
## (ν
## X
## )÷2
φ
## B
mod 64
## 
## )
158rot_r_64_imm1∀i∈N
## 64
## ∶B
## 8
## (φ
## ′
## A
## )
i
## =B
## 8
## (φ
## B
## )
## (i+ν
## X
## )mod 64
159rot_r_64_imm_alt1∀i∈N
## 64
## ∶B
## 8
## (φ
## ′
## A
## )
i
## =B
## 8
## (ν
## X
## )
## (i+φ
## B
## )mod 64
## 160rot_r_32_imm1φ
## ′
## A
## =X
## 4
(x)wherex∈N
## 2
## 32
,∀i∈N
## 32
## ∶B
## 4
## (x)
i
## =B
## 4
## (φ
## B
## )
## (i+ν
## X
## )mod 32
## 161rot_r_32_imm_alt1φ
## ′
## A
## =X
## 4
(x)wherex∈N
## 2
## 32
,∀i∈N
## 32
## ∶B
## 4
## (x)
i
## =B
## 4
## (ν
## X
## )
## (i+φ
## B
## )mod 32
A.5.11.Instructions with Arguments of Two Registers & One Offset.
## (A.30)
letr
## A
## =min(12,(ζ
ı+1
)mod 16),   φ
## A
## ≡φ
r
## A
,  φ
## ′
## A
## ≡φ
## ′
r
## A
letr
## B
## =min(12,
ζ
ı+1
## 16
## ),φ
## B
## ≡φ
r
## B
,  φ
## ′
## B
## ≡φ
## ′
r
## B
letl
## X
## =min(4,max(0,ℓ−1)),ν
## X
≡ı+Z
l
## X
## (E
## −1
l
## X
## (ζ
ı+2⋅⋅⋅+l
## X
## ))
ζ
ı
## Nameρ
## ∆
## Mutations
## 170branch_eq1branch(ν
## X
## ,φ
## A
## =φ
## B
## )
## 171branch_ne1branch(ν
## X
## ,φ
## A
## ≠φ
## B
## )
## 172branch_lt_u1branch(ν
## X
## ,φ
## A
## <φ
## B
## )
## 173branch_lt_s1branch(ν
## X
## ,Z
## 8
## (φ
## A
## )<Z
## 8
## (φ
## B
## ))
## 174branch_ge_u1branch(ν
## X
## ,φ
## A
## ≥φ
## B
## )
## 175branch_ge_s1branch(ν
## X
## ,Z
## 8
## (φ
## A
## )≥Z
## 8
## (φ
## B
## ))
A.5.12.Instruction with Arguments of Two Registers and Two Immediates.
## (A.31)
letr
## A
## =min(12,(ζ
ı+1
## )mod 16),φ
## A
## ≡φ
r
## A
,  φ
## ′
## A
## ≡φ
## ′
r
## A
letr
## B
## =min(12,
ζ
ı+1
## 16
## ),φ
## B
## ≡φ
r
## B
,  φ
## ′
## B
## ≡φ
## ′
r
## B
letl
## X
## =min(4,ζ
ı+2
mod 8),ν
## X
## =X
l
## X
## 
## E
## −1
l
## X
## (ζ
ı+3⋅⋅⋅+l
## X
## )
## 
letl
## Y
## =min(4,max(0,ℓ−l
## X
−2)),   ν
## Y
## =X
l
## Y
## 
## E
## −1
l
## Y
## (ζ
ı+3+l
## X
## ⋅⋅⋅+l
## Y
## )
## 
ζ
ı
## Nameρ
## ∆
## Mutations
## 180load_imm_jump_ind1djump((φ
## B
## +ν
## Y
## )mod 2
## 32
),   φ
## ′
## A
## =ν
## X
A.5.13.Instructions with Arguments of Three Registers.
## (A.32)
letr
## A
## =min(12,(ζ
ı+1
)mod 16),   φ
## A
## ≡φ
r
## A
,  φ
## ′
## A
## ≡φ
## ′
r
## A
letr
## B
## =min(12,
ζ
ı+1
## 16
## ),φ
## B
## ≡φ
r
## B
,  φ
## ′
## B
## ≡φ
## ′
r
## B
letr
## D
## =min(12,ζ
ı+2
## ),φ
## D
## ≡φ
r
## D
,  φ
## ′
## D
## ≡φ
## ′
r
## D

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202543
ζ
ı
## Nameρ
## ∆
## Mutations
## 190add_321φ
## ′
## D
## =X
## 4
## 
## (φ
## A
## +φ
## B
## )mod 2
## 32
## 
## 191sub_321φ
## ′
## D
## =X
## 4
## 
## (φ
## A
## +2
## 32
## −(φ
## B
mod 2
## 32
## ))mod 2
## 32
## 
## 192mul_321φ
## ′
## D
## =X
## 4
## 
## (φ
## A
## ⋅φ
## B
## )mod 2
## 32
## 
## 193div_u_321φ
## ′
## D
## =
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## 2
## 64
## −1ifφ
## B
mod 2
## 32
## =0
## X
## 4
## 
## (φ
## A
mod 2
## 32
## )÷(φ
## B
mod 2
## 32
## )
## 
otherwise
## 194div_s_321φ
## ′
## D
## =
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 2
## 64
## −1ifb=0
## Z
## −1
## 8
## (a)ifa=−2
## 31
## ∧b=−1
## Z
## −1
## 8
## (rtz(a÷b))otherwise
wherea=Z
## 4
## (φ
## A
mod 2
## 32
), b=Z
## 4
## (φ
## B
mod 2
## 32
## )
## 195rem_u_321φ
## ′
## D
## =
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## X
## 4
## 
φ
## A
mod 2
## 32
## 
ifφ
## B
mod 2
## 32
## =0
## X
## 4
## 
## (φ
## A
mod 2
## 32
## )mod(φ
## B
mod 2
## 32
## )
## 
otherwise
## 196rem_s_321φ
## ′
## D
## =
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 0ifa=−2
## 31
## ∧b=−1
## Z
## −1
## 8
## (smod(a,b))otherwise
wherea=Z
## 4
## (φ
## A
mod 2
## 32
), b=Z
## 4
## (φ
## B
mod 2
## 32
## )
## 197shlo_l_321φ
## ′
## D
## =X
## 4
## 
## (φ
## A
## ⋅2
φ
## B
mod 32
## )mod 2
## 32
## 
## 198shlo_r_321φ
## ′
## D
## =X
## 4
## 
## (φ
## A
mod 2
## 32
## )÷2
φ
## B
mod 32
## 
## 199shar_r_321φ
## ′
## D
## =Z
## −1
## 8
## (

## Z
## 4
## (φ
## A
mod 2
## 32
## )÷2
φ
## B
mod 32
## 
## )
## 200add_641φ
## ′
## D
## =(φ
## A
## +φ
## B
## )mod 2
## 64
## 201sub_641φ
## ′
## D
## =(φ
## A
## +2
## 64
## −φ
## B
## )mod 2
## 64
## 202mul_641φ
## ′
## D
## =(φ
## A
## ⋅φ
## B
## )mod 2
## 64
## 203div_u_641φ
## ′
## D
## =
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## 2
## 64
## −1ifφ
## B
## =0
## ⌊φ
## A
## ÷φ
## B
## ⌋otherwise
## 204div_s_641φ
## ′
## D
## =
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 2
## 64
## −1ifφ
## B
## =0
φ
## A
ifZ
## 8
## (φ
## A
## )=−2
## 63
## ∧Z
## 8
## (φ
## B
## )=−1
## Z
## −1
## 8
(rtz(Z
## 8
## (φ
## A
## )÷Z
## 8
## (φ
## B
## )))otherwise
## 205rem_u_641φ
## ′
## D
## =
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
φ
## A
ifφ
## B
## =0
φ
## A
modφ
## B
otherwise
## 206rem_s_641φ
## ′
## D
## =
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
0ifZ
## 8
## (φ
## A
## )=−2
## 63
## ∧Z
## 8
## (φ
## B
## )=−1
## Z
## −1
## 8
(smod(Z
## 8
## (φ
## A
## ),Z
## 8
## (φ
## B
## )))otherwise
## 207shlo_l_641φ
## ′
## D
## =(φ
## A
## ⋅2
φ
## B
mod 64
## )mod 2
## 64
## 208shlo_r_641φ
## ′
## D
## =

φ
## A
## ÷2
φ
## B
mod 64
## 
## 209shar_r_641φ
## ′
## D
## =Z
## −1
## 8
## (

## Z
## 8
## (φ
## A
## )÷2
φ
## B
mod 64
## 
## )
210and1∀i∈N
## 64
## ∶B
## 8
## (φ
## ′
## D
## )
i
## =B
## 8
## (φ
## A
## )
i
## ∧B
## 8
## (φ
## B
## )
i
211xor1∀i∈N
## 64
## ∶B
## 8
## (φ
## ′
## D
## )
i
## =B
## 8
## (φ
## A
## )
i
## ⊕B
## 8
## (φ
## B
## )
i
212or1∀i∈N
## 64
## ∶B
## 8
## (φ
## ′
## D
## )
i
## =B
## 8
## (φ
## A
## )
i
## ∨B
## 8
## (φ
## B
## )
i
## 213mul_upper_s_s1φ
## ′
## D
## =Z
## −1
## 8
## (

## (Z
## 8
## (φ
## A
## )⋅Z
## 8
## (φ
## B
## ))÷2
## 64
## 
## )
## 214mul_upper_u_u1φ
## ′
## D
## =

## (φ
## A
## ⋅φ
## B
## )÷2
## 64
## 
## 215mul_upper_s_u1φ
## ′
## D
## =Z
## −1
## 8
## (

## (Z
## 8
## (φ
## A
## )⋅φ
## B
## )÷2
## 64
## 
## )
## 216set_lt_u1φ
## ′
## D
## =φ
## A
## <φ
## B
## 217set_lt_s1φ
## ′
## D
## =Z
## 8
## (φ
## A
## )<Z
## 8
## (φ
## B
## )
## 218cmov_iz1φ
## ′
## D
## =
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
φ
## A
ifφ
## B
## =0
φ
## D
otherwise

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202544
ζ
ı
## Nameρ
## ∆
## Mutations
## 219cmov_nz1φ
## ′
## D
## =
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
φ
## A
ifφ
## B
## ≠0
φ
## D
otherwise
220rot_l_641∀i∈N
## 64
## ∶B
## 8
## (φ
## ′
## D
## )
## (i+φ
## B
## )mod 64
## =B
## 8
## (φ
## A
## )
i
## 221rot_l_321φ
## ′
## D
## =X
## 4
(x)wherex∈N
## 2
## 32
,∀i∈N
## 32
## ∶B
## 4
## (x)
## (i+φ
## B
## )mod 32
## =B
## 4
## (φ
## A
## )
i
222rot_r_641∀i∈N
## 64
## ∶B
## 8
## (φ
## ′
## D
## )
i
## =B
## 8
## (φ
## A
## )
## (i+φ
## B
## )mod 64
## 223rot_r_321φ
## ′
## D
## =X
## 4
(x)wherex∈N
## 2
## 32
,∀i∈N
## 32
## ∶B
## 4
## (x)
i
## =B
## 4
## (φ
## A
## )
## (i+φ
## B
## )mod 32
224and_inv1∀i∈N
## 64
## ∶B
## 8
## (φ
## ′
## D
## )
i
## =B
## 8
## (φ
## A
## )
i
## ∧ ¬B
## 8
## (φ
## B
## )
i
225or_inv1∀i∈N
## 64
## ∶B
## 8
## (φ
## ′
## D
## )
i
## =B
## 8
## (φ
## A
## )
i
## ∨ ¬B
## 8
## (φ
## B
## )
i
226xnor1∀i∈N
## 64
## ∶B
## 8
## (φ
## ′
## D
## )
i
## =¬(B
## 8
## (φ
## A
## )
i
## ⊕B
## 8
## (φ
## B
## )
i
## )
## 227max1φ
## ′
## D
## =Z
## −1
## 8
(max(Z
## 8
## (φ
## A
## ),Z
## 8
## (φ
## B
## )))
## 228max_u1φ
## ′
## D
## =max(φ
## A
## ,φ
## B
## )
## 229min1φ
## ′
## D
## =Z
## −1
## 8
(min(Z
## 8
## (φ
## A
## ),Z
## 8
## (φ
## B
## )))
## 230min_u1φ
## ′
## D
## =min(φ
## A
## ,φ
## B
## )
Note that the two signed modulo operations have an idiosyncratic definition, operating as the modulo of the absolute
values, but with the sign of the numerator. Formally:
(A.33)smod∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## Z,Z
## ⎫
## ⎭
## →Z
## (a,b)↦
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
aifb=0
sgn(a)⋅(SaSmodSbS)otherwise
Division operations always round their result towards zero. Formally:
## (A.34)
rtz∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## Z→Z
x↦
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## ⌈x⌉ifx<0
## ⌊x⌋otherwise
A.6.Host Call Definition.An extended version of thepvminvocation which is able to progress an innerhost-call
state-machine in the case of a host-call halt condition is defined asΨ
## H
## :
## Ψ
## H
## ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## B,N
## R
## ,N
## G
## ,⟦N
## R
## ⟧
## 13
## ,
## M,Ω⟨X⟩,X
## ⎫
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎭
## →
## ⎧
## ⎪
## ⎩
## 
## ☇,∞,∎
## 
## ∪{
## F
## }×N
## R
## ,N
## R
## ,Z
## G
## ,⟦N
## R
## ⟧
## 13
## ,M,X
## ⎫
## ⎪
## ⎭
## (c,ı,ρ,φ,μ,f,x)↦
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
let(ε
## ′
## ,ı
## ′
## ,ρ
## ′
## ,φ
## ′
## ,μ
## ′
)=Ψ(c,ı,ρ,φ,μ)∶
## (ε
## ′
## ,ı
## ′
## ,ρ
## ′
## ,φ
## ′
## ,μ
## ′
## ,x)ifε
## ′
## ∈
## 
## ∎,☇,∞
## 
## ∪{
## F
## }×N
## R
## Ψ
## H
## (c,ı
## ′′
## ,ρ
## ′′
## ,φ
## ′′
## ,μ
## ′′
## ,f,x
## ′′
## )
whereı
## ′′
## =ı
## ′
## +1+skip(ı
## ′
## )
if
## ⋀
## 
ε
## ′
## =
## ̵
h×h
## 
## ▸,ρ
## ′′
## ,φ
## ′′
## ,μ
## ′′
## ,x
## ′′
## 
## =f(h,ρ
## ′
## ,φ
## ′
## ,μ
## ′
## ,x)
## (ε
## ′′
## ,ı
## ′
## ,ρ
## ′′
## ,φ
## ′′
## ,μ
## ′′
## ,x
## ′′
## )if
## ⋀
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
ε
## ′
## =
## ̵
h×h
## 
ε
## ′′
## ,ρ
## ′′
## ,φ
## ′′
## ,μ
## ′′
## ,x
## ′′
## 
## =f(h,ρ
## ′
## ,φ
## ′
## ,μ
## ′
## ,x)
ε
## ′′
## ∈
## 
## ☇,∎,∞
## 
## (A.35)
## Ω⟨X⟩≡
## ⎧
## ⎩
## N,N
## G
## ,⟦N
## R
## ⟧
## 13
## ,M,X
## ⎫
## ⎭
## →
## ⎧
## ⎪
## ⎩
## 
## ▸,∎,☇,∞
## 
## ,N
## G
## ,⟦N
## R
## ⟧
## 13
## ,M,X
## ⎫
## ⎪
## ⎭
## (A.36)
As withΦ, on exit the instruction counter references the instructionwhich caused the exitand the machine state is
that prior to this instruction. Should the machine be invoked again using this instruction counter and code, then the
same instruction which caused the exit would be executed on the proper (prior) machine state.
WithΦ
## H
, host-calls (i.e.ecalliinstructions) are in effect handled internally with the state-mutator function provided
as an argument, preventing the possibility of the result being a host-call fault. Note that in the case of a successful
host-call transition, we must provide the new instruction counter valueı
## ′′
explicitly alongside the fresh posterior state
for said instruction.
A.7.Standard Program Initialization.The software programs which will run in each of the four instances where
thepvmis utilized in the main document have a very typical setup pattern characteristic of an output of a compiler and
linker. This means thatramhas sections for program-specific read-only data, read-write (heap) data and the stack. An
adjunct to this, very typical of our usage patterns is an extra read-only section via which invocation-specific data may
be passed (i.e. arguments). It thus makes sense to define this properly in a single initializer function. These sections are

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202545
quantized intomajor zones, and one major zone is always left unallocated between sections in order to reduce accidental
overrun. Sections are padded with zeroes to the nearestpvmmemory page boundary.
We thus define the standard program code formatp, which includes not only the instructions and jump table (previ-
ously represented by the termc), but also information on the state of theramat program start. Given program blobp
and argument dataa, we can decode the program codec, registersφ, andramμby invoking the standard initialization
functionY(p,a):
## (A.37)
## Y∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## B,B
## ∶Z
## I
## ⎫
## ⎭
## →
## ⎧
## ⎩
## B,⟦N
## R
## ⟧
## 13
## ,M
## ⎫
## ⎭
## ?
## (p,a)↦
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
(c,φ,μ)if∃!(c,o,w,z,s)which satisfy equationA.38
## ∅otherwise
With conditions:
letE
## 3
(SoS)⌢E
## 3
(SwS)⌢E
## 2
(z)⌢E
## 3
(s)⌢o⌢w⌢E
## 4
(ScS)⌢c=p(A.38)
## Z
## Z
## =2
## 16
## ,Z
## I
## =2
## 24
## (A.39)
letP(x∈N)≡Z
## P
## 
x
## Z
## P
,   Z(x∈N)≡Z
## Z
## 
x
## Z
## Z
## (A.40)
## 5Z
## Z
+Z(SoS)+Z(SwS+zZ
## P
)+Z(s)+Z
## I
## ≤2
## 32
## (A.41)
Thus, if the above conditions cannot be satisfied with unique values, then the result is∅, otherwise it is a tuple ofcas
above andμ,φsuch that:
(A.42)∀i∈N
## 2
## 32
## ∶((μ
v
## )
i
## ,(μ
a
## )
## ⌊
i
## ~Z
## P
## ⌋
## )=
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## (v
## ▸
## ▸
o
i−Z
## Z
## ,a
## ▸
## ▸
R)ifZ
## Z
≤i<Z
## Z
+SoS
(0,R)ifZ
## Z
+SoS≤i<Z
## Z
+P(SoS)
## (w
i−(2Z
## Z
+Z(SoS))
,W)if2Z
## Z
+Z(SoS)≤i<2Z
## Z
+Z(SoS)+SwS
(0,W)if2Z
## Z
+Z(SoS)+SwS≤i<2Z
## Z
+Z(SoS)+P(SwS)+zZ
## P
(0,W)if2
## 32
## −2Z
## Z
## −Z
## I
−P(s)≤i<2
## 32
## −2Z
## Z
## −Z
## I
## (a
i−(2
## 32
## −Z
## Z
## −Z
## I
## )
,R)if2
## 32
## −Z
## Z
## −Z
## I
## ≤i<2
## 32
## −Z
## Z
## −Z
## I
+SaS
(0,R)if2
## 32
## −Z
## Z
## −Z
## I
+SaS≤i<2
## 32
## −Z
## Z
## −Z
## I
+P(SaS)
## (0,∅)otherwise
## (A.43)
∀i∈N
## 13
## ∶φ
i
## =
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 2
## 32
## −2
## 16
ifi=0
## 2
## 32
## −2Z
## Z
## −Z
## I
ifi=1
## 2
## 32
## −Z
## Z
## −Z
## I
ifi=7
SaSifi=8
## 0otherwise
A.8.Argument Invocation Definition.The four instances where thepvmis utilized each expect to be able to pass
argument data in and receive some return data back. We thus define the commonpvmprogram-argument invocation
functionΨ
## M
## :
## (A.44)Ψ
## M
## ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## B,N
## R
## ,N
## G
## ,B
## ∶Z
## I
## ,Ω⟨X⟩,X
## ⎫
## ⎭
## →
## ⎧
## ⎪
## ⎩
## N
## G
## ,B∪
## 
## ☇,∞
## 
## ,X
## ⎫
## ⎪
## ⎭
## (p,ı,ρ,a,f,x)↦
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## 0,☇,x
## 
ifY(p,a)=∅
R(ρ,Ψ
## H
(c,ı,ρ,φ,μ,f,x))ifY(p,a)=(c,φ,μ)
whereR∶ρ,
ε, ı
## ′
, ρ
## ′
## ,
φ
## ′
## ,μ
## ′
## ,x
## ′
## ↦
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## (u,∞,x
## ′
## )ifε=∞
## u,μ
## ′
φ
## ′
## 7
## ⋅⋅⋅+φ
## ′
## 8
## ,x
## ′
ifε=∎ ∧N
φ
## ′
## 7
## ⋅⋅⋅+φ
## ′
## 8
## ⊆V
μ
## ′
## (u,[],x
## ′
)ifε=∎ ∧N
φ
## ′
## 7
## ⋅⋅⋅+φ
## ′
## 8
## ~⊆V
μ
## ′
## 
u,☇,x
## ′
## 
otherwise
whereu=ρ−max(ρ
## ′
## ,0)
Note that the first tuple item is the amount of gas consumed by the operation, but never greater than the amount of
gas provided for the operation.
AppendixB.Virtual Machine Invocations
We now define the three practical instances where we wish to invoke apvminstance as part of the protocol. In
general, we avoid introducing unbounded data as part of the basic invocation arguments in order to minimize the chance
of an unexpectedly largeramallocation, which could lead to gas inflation and unavoidable underflow. This makes for a
more cumbersome interface, but one which is more predictable and easier to reason about.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202546
B.1.Host-Call Result Constants.
## NONE=2
## 64
−1:The return value indicating an item does not exist.
## WHAT=2
## 64
−2:Name unknown.
## OOB=2
## 64
−3:The innerpvmmemory index provided for reading/writing is not accessible.
## WHO=2
## 64
−4:Index unknown.
## FULL=2
## 64
−5:Storage full or resource already allocated.
## CORE=2
## 64
−6:Core index unknown.
## CASH=2
## 64
−7:Insuﬀicient funds.
## LOW=2
## 64
−8:Gas limit too low.
## HUH=2
## 64
−9:The item is already solicited, cannot be forgotten or the operation is invalid due to privilege level.
OK=0:The return value indicating general success.
Innerpvminvocations have their own set of result codes:
HALT=0:The invocation completed and halted normally.
PANIC=1:The invocation completed with a panic.
FAULT=2:The invocation completed with a page fault.
HOST=3:The invocation completed with a host-call fault.
OOG=4:The invocation completed by running out of gas.
Note return codes for a host-call-request exit are any non-zero value less than2
## 64
## −13.
B.2.Is-Authorized Invocation.The Is-Authorized invocation is the first and simplest of the four, being totally
stateless. It provides only host-call functions for inspecting its environment and parameters. It accepts as arguments
only the core on which it should be executed,c. Formally, it is defined asΨ
## I
## :
## Ψ
## I
## ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## P,N
## C
## ⎫
## ⎭
## →
## ⎧
## ⎩
## B∪E,N
## G
## ⎫
## ⎭
## (p,c)↦
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
(BAD,0)ifp
u
## =∅
(BIG,0)otherwise ifSp
u
## S>W
## A
## (r,u)otherwise
where(u,r,∅)=Ψ
## M
## (p
u
## ,0,G
## I
## ,E
## 2
(c),F,∅)
## (B.1)
F∈Ω⟨{}⟩∶(n,ρ,φ,μ)↦
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## Ω
## G
## (ρ,φ,μ)ifn=gas
## Ω
## Y
## (ρ,φ,μ,p,∅,∅,∅,∅,∅,∅,∅)ifn=fetch
## (∞,ρ
## ′
## ,φ
## ′
,μ)otherwise ifρ
## ′
## <0
## (▸,ρ
## ′
## ,φ
## ′
## ,μ)otherwise
whereφ
## ′
## =φexceptφ
## ′
## 7
## =WHAT
andρ
## ′
## =ρ−10
## (B.2)
Note for the Is-Authorized host-call dispatch functionFin equationB.2, we elide the host-call context since, being
essentially stateless, it is always∅.
B.3.Refine Invocation.We define the Refine service-account invocation function asΨ
## R
. It has no general access to
the state of the
## J
amchain, with the slight exception being the ability to make a historical lookup. Beyond this it is able
to create inner instances of thepvmand dictate pieces of data to export.
The historical-lookup host-call function,Ω
## H
, is designed to give the same result regardless of the state of the chain for
any time when auditing may occur (which we bound to be less than two epochs from being accumulated). The lookup
anchor may be up toLtimeslots before the recent history and therefore adds to the potential age at the time of audit.
We therefore setDto have a safety margin of eight hours:
## (B.3)D≡L+4,800=19,200
The innerpvminvocation host-calls, meanwhile, depend on an integratedpvmtype, which we shall denoteG. It holds
some program code, instruction counter andram:
## (B.4)G≡
## ⎧
## ⎩
p∈B,u∈M,i∈N
## R
## ⎫
## ⎭
The Export host-call depends on two pieces of context; one sequence of segments (blobs of lengthW
## G
) to which it
may append, and the other an argument passed to the invocation function to dictate the number of segments prior which
may assumed to have already been appended. The latter value ensures that an accurate segment index can be provided
to the caller.
Unlike the other invocation functions, the Refine invocation function implicitly draws upon some recent service account
state itemδ. The specific block from which this comes is not important, as long as it is no earlier than its work-package’s
lookup-anchor block. It explicitly accepts the work-packagepand the index of the work item to be refined,itogether
with the core which is doing the refiningc. Additionally, the authorizer traceris provided together with all work items’
import segments
iand an export segment offsetς. It results in a tuple of some errorEor the refinement output blob

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202547
(signalling success), the export sequence in the case of success and the gas used in evaluation. Formally:
## Ψ
## R
## ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎪
## ⎩
## N
## C
## ,N,P,B,
## C
## ⟦J⟧
## H
## ,N
## ⎫
## ⎪
## ⎭
## →
## ⎧
## ⎩
## B∪E,⟦J⟧,N
## G
## ⎫
## ⎭
## 
c,i,p,r,
i,ς
## 
## ↦
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
(BAD,[],0)ifw
s
~∈K(δ)∨Λ(δ[w
s
## ],(p
c
## )
t
## ,w
c
## )=∅
(BIG,[],0)otherwise ifSΛ(δ[w
s
## ],(p
c
## )
t
## ,w
c
## )S>W
## C
otherwise∶
leta=E(c,i,w
s
## ,↕w
y
,H(p)),E(↕z,c)=Λ(δ[w
s
## ],(p
c
## )
t
## ,w
c
## )
and(u,o,(m,e))=Ψ
## M
## (c,0,w
g
,a,F,(∅,[]))∶
## (o,[],u)ifo∈
## 
## ∞,☇
## 
## (o,e,u)otherwise
wherew=p
w
## [i]
## (B.5)
## F∈Ω
a
## ⎧
## ⎩
jN→Go,⟦J⟧
## ⎫
## ⎭
f
## ∶(n,ρ,φ,μ,(m,e))↦
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## Ω
## G
## (ρ,φ,μ,(m,e))ifn=gas
## Ω
## Y
(ρ,φ,μ,p,H
## 0
## ,r,i,
i,x,∅,(m,e))ifn=fetch
## Ω
## H
## (ρ,φ,μ,(m,e),w
s
## ,δ,(p
c
## )
t
## )ifn=historical_lookup
## Ω
## E
## (ρ,φ,μ,(m,e),ς)ifn=export
## Ω
## M
## (ρ,φ,μ,(m,e))ifn=machine
## Ω
## P
## (ρ,φ,μ,(m,e))ifn=peek
## Ω
## O
## (ρ,φ,μ,(m,e))ifn=poke
## Ω
## Z
## (ρ,φ,μ,(m,e))ifn=pages
## Ω
## K
## (ρ,φ,μ,(m,e))ifn=invoke
## Ω
## X
## (ρ,φ,μ,(m,e))ifn=expunge
## (∞,ρ
## ′
## ,φ
## ′
,μ)otherwise ifρ
## ′
## <0
## (▸,ρ
## ′
## ,φ
## ′
## ,μ)otherwise
whereφ
## ′
## =φexceptφ
## ′
## 7
## =WHAT
andρ
## ′
## =ρ−10
andx=[[xS (H(x),SxS)<−w
x
## ] Sw<−p
w
## ]
## (B.6)
B.4.Accumulate Invocation.Since this is a transition which can directly affect a substantial amount of on-chain
state, our invocation context is accordingly complex. It is a tuple with elements for each of the aspects of state which
can be altered through this invocation and beyond the account of the service itself includes the deferred transfer list and
several dictionaries for alterations to preimage lookup state, core assignments, validator key assignments, newly created
accounts and alterations to account privilege levels.
Formally, we define our result context to beL, and our invocation context to be a pair of these contexts,L×L(and
thus for any valuex∈Lthere existsx
## 2
∈L×L), with one dimension being the regular dimension and generally namedx
and the other being the exceptional dimension and being namedy. The only function which actually alters this second
dimension ischeckpoint,Ω
## C
and so it is rarely seen.
## L≡
## ⎧
## ⎩
s∈N
## S
,e∈S,i∈N
## S
,t∈⟦X⟧,y∈H?,p∈{[
## ⎧
## ⎩
## N
## S
## ,B
## ⎫
## ⎭
## ]}
## ⎫
## ⎭
## (B.7)
∀x∈L∶x
s
## ≡(x
e
## )
d
## [x
s
## ](B.8)
We define a convenience equivalencex
s
to easily denote the accumulating service account.
We track both regular and exceptional dimensions within our context mutator, but collapse the result of the invocation
to one or the other depending on whether the termination was regular or exceptional (i.e. out-of-gas or panic).
We defineΨ
## A
, the Accumulation invocation function as:
## Ψ
## A
## ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## S,N
## T
## ,N
## S
## ,N
## G
## ,⟦I⟧
## ⎫
## ⎭
## →O
## (e,t,s,g,i)↦
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## (e
## ▸
## ▸
s,t
## ▸
## ▸
## [],y
## ▸
## ▸
## ∅,u
## ▸
## ▸
## 0,p
## ▸
## ▸
[])ifc=∅ ∨ScS>W
## C
## C(Ψ
## M
(c,5,g,E(t,s,SiS),F,I(s,s)
## 2
## ))otherwise
wherec=e
d
## [s]
c
ands=eexcepts
d
## [s]
b
## =e
d
## [s]
b
## +
## ∑
r∈x
r
a
andx=[iSi<−i,i∈X]
## (B.9)
## I∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## S,N
## S
## ⎫
## ⎭
## →L
## (e,s)↦(s,e,i,t
## ▸
## ▸
## [],y
## ▸
## ▸
## ∅,p
## ▸
## ▸
## [])
wherei=check((E
## −1
## 4
## 
## H
## 
## E
## 
s,η
## ′
## 0
## ,H
## T
## 
mod(2
## 32
## −S−2
## 8
## ))+S)
## (B.10)

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202548
## F∈Ω⟨
## ⎧
## ⎩
## L,L
## ⎫
## ⎭
## ⟩∶(n,ρ,φ,μ,(x,y))↦
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## Ω
## G
## (ρ,φ,μ,(x,y))ifn=gas
## Ω
## Y
## (ρ,φ,μ,∅,η
## ′
## 0
## ,∅,∅,∅,∅,i,(x,y))ifn=fetch
## G(Ω
## R
## (ρ,φ,μ,x
s
## ,x
s
## ,(x
e
## )
d
## ),(x,y))ifn=read
## G(Ω
## W
## (ρ,φ,μ,x
s
## ,x
s
## ),(x,y))ifn=write
## G(Ω
## L
## (ρ,φ,μ,x
s
## ,x
s
## ,(x
e
## )
d
## ),(x,y))ifn=lookup
## G(Ω
## I
## (ρ,φ,μ,x
s
## ,(x
e
## )
d
## ),(x,y))ifn=info
## Ω
## B
## (
ρ,φ,μ,
## (
x
## ,
y
## ))
if
n
## =
bless
## Ω
## A
## (ρ,φ,μ,(x,y))ifn=assign
## Ω
## D
## (ρ,φ,μ,(x,y))ifn=designate
## Ω
## C
## (ρ,φ,μ,(x,y))ifn=checkpoint
## Ω
## N
(ρ,φ,μ,(x,y),H
## T
## )ifn=new
## Ω
## U
## (ρ,φ,μ,(x,y))ifn=upgrade
## Ω
## T
## (ρ,φ,μ,(x,y))ifn=transfer
## Ω
## J
(ρ,φ,μ,(x,y),H
## T
## )ifn=eject
## Ω
## Q
## (ρ,φ,μ,(x,y))ifn=query
## Ω
## S
(ρ,φ,μ,(x,y),H
## T
## )ifn=solicit
## Ω
## F
(ρ,φ,μ,(x,y),H
## T
## )ifn=forget
## Ω
## Q
## (
ρ,φ,μ,
## (
x
## ,
y
## ))
if
n
## =
yield
## Ω
## P
## (ρ,φ,μ,(x,y))ifn=provide
## (∞,ρ
## ′
## ,φ
## ′
,μ,(x,y))otherwise ifρ
## ′
## <0
## (▸,ρ
## ′
## ,φ
## ′
## ,μ,(x,y))otherwise
whereφ
## ′
## =φexceptφ
## ′
## 7
## =WHAT
andρ
## ′
## =ρ−10
## (B.11)
## G∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎪
## ⎩
## ⎧
## ⎪
## ⎩
## 
## ▸,∎,☇,∞
## 
## ,N
## G
## ,⟦N
## R
## ⟧
## 13
## ,M,A
## ⎫
## ⎪
## ⎭
## ,
## ⎧
## ⎩
## L,L
## ⎫
## ⎭
## ⎫
## ⎪
## ⎭
## →
## ⎧
## ⎪
## ⎩
## 
## ▸,∎,☇,∞
## 
## ,N
## G
## ,⟦N
## R
## ⟧
## 13
## ,M,
## ⎧
## ⎩
## L,L
## ⎫
## ⎭
## ⎫
## ⎪
## ⎭
## ((ε,ρ,φ,μ,s),(x,y))↦
## 
ε,ρ,φ,μ,
## 
x
## ∗
## ,y
## 
wherex
## ∗
## =xexceptx
## ∗
s
## =s
## (B.12)
## C∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎪
## ⎩
## N
## G
## ,B∪
## 
## ∞,☇
## 
## ,
## ⎧
## ⎩
## L,L
## ⎫
## ⎭
## ⎫
## ⎪
## ⎭
## →O
## (u,o,(x,y))↦
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## (e
## ▸
## ▸
y
e
## ,t
## ▸
## ▸
y
t
## ,y
## ▸
## ▸
y
y
## ,u,p
## ▸
## ▸
y
p
## )ifo∈
## 
## ∞,☇
## 
## 
e
## ▸
## ▸
x
e
## ,t
## ▸
## ▸
x
t
## ,y
## ▸
## ▸
o,u,p
## ▸
## ▸
## (x,y)
p
## 
otherwise ifo∈H
## (e
## ▸
## ▸
x
e
## ,t
## ▸
## ▸
x
t
## ,y
## ▸
## ▸
x
y
## ,u,p
## ▸
## ▸
x
p
## )otherwise
## (B.13)
The mutatorFgoverns how this context will alter for any given parameterization, and the collapse functionCselects
one of the two dimensions of context depending on whether the virtual machine’s halt was regular or exceptional.
The initializer functionImaps some partial state along with a service account index to yield a mutator context such
that no alterations to the given state are implied in either exit scenario. Note that the componentautilizes the random
accumulatorη
## ′
## 0
and the block’s timeslotH
## T
to create a deterministic sequence of identifiers which are extremely likely
to be unique.
Concretely, we create the identifier from the Blake2 hash of the identifier of the creating service, the current random
accumulatorη
## ′
## 0
and the block’s timeslot. Thus, within a service’s accumulation it is almost certainly unique, but it is
not necessarily unique across all services, nor at all times in the past. We utilize acheckfunction to find the first such
index in this sequence which does not already represent a service:
(B.14)check(i∈N
## S
## )≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
iifi~∈K(e
d
## )
check((i−S+1)mod(2
## 32
## −2
## 8
−S)+S)otherwise
nbIn the highly unlikely event that a block executes to find that a single service index has inadvertently been attached
to two different services, then the block is considered invalid. Since no service can predict the identifier sequence ahead
of time, they cannot intentionally disadvantage the block author.
B.5.General Functions.We come now to defining the host functions which are utilized by thepvminvocations.
Generally, these map somepvmstate, including invocation context, possibly together with some additional parameters,
to a newpvmstate.
The general functions are all broadly of the form
## 
ρ
## ′
## ∈Z
## G
## ,φ
## ′
## ∈⟦N
## R
## ⟧
## 13
## ,μ
## ′
## ∈M
## 
## =Ω
## ◻
(ρ∈N
## G
,φ∈⟦N
## R
## ⟧
## 13
,μ∈M).
Functions which have a result component which is equivalent to the corresponding argument may have said components
elided in the description. Functions may also depend upon particular additional parameters.
Unlike the Accumulate functions in appendix
B.7, these do not mutate an accumulation context. Some, such as
write
mutate a service account and both accept and return somes∈A. Others are more general functions, such asfetchand

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202549
do not assume any context but have a parameter list suﬀixed with an ellipsis to denote that the context parameter may
be taken and is provided transparently into its result. This allows it to be easily utilized in multiplepvminvocations.
Other than the gas-counter which is explicitly defined, elements ofpvmstate are each assumed to remain unchanged
by the host-call unless explicitly specified.
ρ
## ′
≡ρ−g(B.15)
## 
ε
## ′
## ,φ
## ′
## ,μ
## ′
## ,s
## ′
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## (∞,φ,μ,s)ifρ<g
(▸,φ,μ,s)except as indicated below otherwise
## (B.16)
## Function
## Identifier
Gas usage
## Mutations
## Ω
## G
## (ρ,φ,...)
gas= 0
g=10
φ
## ′
## 7
## ≡ρ
## ′
## Ω
## Y
## (ρ,φ,μ,p,n,r,i,i,x,i,...)
fetch= 1
g=10
letv=
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
cifφ
## 10
## =0
wherec=E
## ⎛
## ⎜
## ⎜
## ⎜
## ⎜
## ⎜
## ⎝
## E
## 8
## (B
## I
## ),E
## 8
## (B
## L
## ),E
## 8
## (B
## S
## ),E
## 2
## (C),E
## 4
## (D),E
## 4
## (E),E
## 8
## (G
## A
## ),
## E
## 8
## (G
## I
## ),E
## 8
## (G
## R
## ),E
## 8
## (G
## T
## ),E
## 2
## (H),E
## 2
## (I),E
## 2
## (J),E
## 2
## (K),
## E
## 4
## (L),E
## 2
## (N),E
## 2
## (O),E
## 2
## (P),E
## 2
## (Q),E
## 2
## (R),E
## 2
## (T),E
## 2
## (U),
## E
## 2
## (V),E
## 4
## (W
## A
## ),E
## 4
## (W
## B
## ),E
## 4
## (W
## C
## ),E
## 4
## (W
## E
## ),E
## 4
## (W
## M
## ),
## E
## 4
## (W
## P
## ),E
## 4
## (W
## R
## ),E
## 4
## (W
## T
## ),E
## 4
## (W
## X
## ),E
## 4
## (Y)
## ⎞
## ⎟
## ⎟
## ⎟
## ⎟
## ⎟
## ⎠
nifn≠∅ ∧φ
## 10
## =1
rifr≠∅ ∧φ
## 10
## =2
x[φ
## 11
## ]
φ
## 12
if
x≠∅ ∧φ
## 10
## =3∧φ
## 11
<SxS∧φ
## 12
<Sx[φ
## 11
## ]S
x[i]
φ
## 11
if
x≠∅ ∧i≠∅ ∧φ
## 10
## =4∧φ
## 11
<Sx[i]S
i[φ
## 11
## ]
φ
## 12
if
i≠∅ ∧φ
## 10
## =5∧φ
## 11
## <
## T
i
## T
## ∧φ
## 12
## <
## T
i[φ
## 11
## ]
## T
i[i]
φ
## 11
ifi≠∅ ∧i≠∅ ∧φ
## 10
## =6∧φ
## 11
## <
## T
i[i]
## T
## E(p)ifp≠∅ ∧φ
## 10
## =7
p
f
ifp≠∅ ∧φ
## 10
## =8
p
j
ifp≠∅ ∧φ
## 10
## =9
## E(p
c
## )ifp≠∅ ∧φ
## 10
## =10
E(↕[S(w) Sw<
## −p
w
## ])ifp≠∅ ∧φ
## 10
## =11
## S(p
w
## [φ
## 11
## ])ifp≠∅ ∧φ
## 10
## =12∧φ
## 11
<Sp
w
## S
whereS(w)≡E(E
## 4
## (w
s
## ),w
c
## ,E
## 8
## (w
g
## ,w
a
## ),E
## 2
## (w
e
,Sw
i
S,Sw
x
## S),E
## 4
(Sw
y
## S))
p
w
## [φ
## 11
## ]
y
ifp≠∅ ∧φ
## 10
## =13∧φ
## 11
<Sp
w
## S
## E(↕i)ifi≠∅ ∧φ
## 10
## =14
## E(i[φ
## 11
## ])ifi≠∅ ∧φ
## 10
## =15∧φ
## 11
<SiS
## ∅otherwise
leto=φ
## 7
letf=min(φ
## 8
,SvS)
letl=min(φ
## 9
,SvS−f)
## 
ε
## ′
## ,φ
## ′
## 7
## ,μ
## ′
o⋅⋅⋅+l
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,μ
o⋅⋅⋅+l
## 
ifN
o⋅⋅⋅+l
## ~⊆V
## ∗
μ
(▸,NONE,μ
o⋅⋅⋅+l
)otherwise ifv=∅
(▸,SvS,v
f⋅⋅⋅+l
## )otherwise

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202550
## Function
## Identifier
Gas usage
## Mutations
## Ω
## L
## (ρ,φ,μ,s,s,d)
lookup= 2
g=10
leta=
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
sifφ
## 7
## ∈
## 
s,2
## 64
## −1
## 
d[φ
## 7
]otherwise ifφ
## 7
∈K(d)
## ∅otherwise
let[h,o]=φ
## 8⋅⋅⋅+2
let
v
## =
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
∇ifN
h⋅⋅⋅+32
## ~⊆V
μ
∅otherwise ifa=∅ ∨μ
h
## ⋅⋅⋅+
## 32
~∈K(a
p
## )
a
p
## [μ
h⋅⋅⋅+32
## ]otherwise
letf=min(φ
## 10
,SvS)
letl=min(φ
## 11
,SvS−f)
## 
ε
## ′
## ,φ
## ′
## 7
## ,μ
## ′
o⋅⋅⋅+l
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,μ
o⋅⋅⋅+l
## 
ifv=∇ ∨N
o⋅⋅⋅+l
## ~⊆V
## ∗
μ
(▸,NONE,μ
o⋅⋅⋅+l
)otherwise ifv=∅
(▸,SvS,v
f⋅⋅⋅+l
## )otherwise
## Ω
## R
## (ρ,φ,μ,s,s,d)
read= 3
g=10
lets
## ∗
## =
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
sifφ
## 7
## =2
## 64
## −1
φ
## 7
otherwise
leta=
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
sifs
## ∗
## =s
d[s
## ∗
]otherwise ifs
## ∗
∈K(d)
## ∅otherwise
let[k
## O
## ,k
## Z
## ,o]=φ
## 8⋅⋅⋅+3
letv=
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
∇ifN
k
## O
## ⋅⋅⋅+k
## Z
## ~⊆V
μ
a
s
[k]otherwise ifa≠∅ ∧k∈K(a
s
## ),wherek=μ
k
## O
## ⋅⋅⋅+k
## Z
## ∅otherwise
letf=min(φ
## 11
,SvS)
letl=min(φ
## 12
,SvS−f)
## 
ε
## ′
## ,φ
## ′
## 7
## ,μ
## ′
o⋅⋅⋅+l
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,μ
o⋅⋅⋅+l
## 
ifv=∇ ∨N
o⋅⋅⋅+l
## ~⊆V
## ∗
μ
(▸,NONE,μ
o⋅⋅⋅+l
)otherwise ifv=∅
(▸,SvS,v
f⋅⋅⋅+l
## )otherwise
## Ω
## W
## (ρ,φ,μ,s,s)
write= 4
g=10
let[k
## O
## ,k
## Z
## ,v
## O
## ,v
## Z
## ]=φ
## 7⋅⋅⋅+4
letk=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
μ
k
## O
## ⋅⋅⋅+k
## Z
ifN
k
## O
## ⋅⋅⋅+k
## Z
## ⊆V
μ
## ∇otherwise
leta=
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
s,exceptK(a
s
)=K(a
s
## )∖{k}ifv
## Z
## =0
s
## ,
except
a
s
## [
k
## ]
## =
μ
v
## O
## ⋅⋅⋅+v
## Z
otherwise ifN
v
## O
## ⋅⋅⋅+v
## Z
## ⊆V
μ
## ∇otherwise
letl=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## Ss
s
[k]Sifk∈K(s
s
## )
NONEotherwise
## 
ε
## ′
## ,φ
## ′
## 7
## ,s
## ′
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,s
## 
ifk=∇ ∨a=∇
(▸,FULL,s)otherwise ifa
t
## >a
b
## (▸,l,a
## )
otherwise

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202551
## Function
## Identifier
Gas usage
## Mutations
## Ω
## I
## (ρ,φ,μ,s,d)
info= 5
g=10
leta=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
d[s]ifφ
## 7
## =2
## 64
## −1
d[φ
## 7
## ]otherwise
leto=φ
## 8
letv=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## E(a
c
## ,E
## 8
## (a
b
## ,a
t
## ,a
g
## ,a
m
## ,a
o
## ),E
## 4
## (a
i
## ),E
## 8
## (a
f
## ),E
## 4
## (a
r
## ,a
a
## ,a
p
## ))ifa≠∅
## ∅otherwise
letf=min(φ
## 9
,SvS)
letl=min(φ
## 10
,SvS−f)
## 
ε
## ′
## ,φ
## ′
## 7
## ,μ
## ′
o⋅⋅⋅+l
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,μ
o⋅⋅⋅+l
## 
ifv=∇ ∨N
o⋅⋅⋅+l
## ~⊆V
## ∗
μ
(▸,NONE,μ
o⋅⋅⋅+l
)otherwise ifv=∅
(▸,SvS,v
f⋅⋅⋅+l
## )otherwise
B.6.Refine Functions.These assume some refine context pair(m,e)∈
## ⎧
## ⎩
jN→Go,⟦J⟧
## ⎫
## ⎭
, which are both initially empty.
Other than the gas-counter which is explicitly defined, elements ofpvmstate are each assumed to remain unchanged by
the host-call unless explicitly specified.
ρ
## ′
≡ρ−g(B.17)
## 
ε
## ′
## ,φ
## ′
## ,μ
## ′
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## (∞,φ,μ)ifρ<g
(▸,φ,μ)except as indicated below otherwise
## (B.18)
## Function
## Identifier
Gas usage
## Mutations
## Ω
## H
## (ρ,φ,μ,(m,e),s,d,t)
historical_lookup= 6
g=10
leta=
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
d[s]ifφ
## 7
## =2
## 64
−1∧s∈K(d)
d[φ
## 7
## ]ifφ
## 7
∈K(d)
## ∅otherwise
let[h,o]=φ
## 8⋅⋅⋅+2
letv=
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
∇ifN
h⋅⋅⋅+32
## ~⊆V
μ
∅otherwise ifa=∅
## Λ(a,t,μ
h⋅⋅⋅+32
## )otherwise
letf=min(φ
## 10
,SvS)
letl=min(φ
## 11
,SvS−f)
## 
ε
## ′
## ,φ
## ′
## 7
## ,μ
## ′
o⋅⋅⋅+l
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,μ
o⋅⋅⋅+l
## 
ifv=∇ ∨N
o⋅⋅⋅+l
## ~⊆V
## ∗
μ
(▸,NONE,μ
o⋅⋅⋅+l
)otherwise ifv=∅
(▸,SvS,v
f⋅⋅⋅+l
## )otherwise
## Ω
## E
## (ρ,φ,μ,(m,e),ς)
export= 7
g=10
letp=φ
## 7
letz=min(φ
## 8
## ,W
## G
## )
letx=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## P
## W
## G
## (μ
p⋅⋅⋅+z
)ifN
p⋅⋅⋅+z
## ⊆V
## [
μ]
## ∇otherwise
## 
ε
## ′
## ,φ
## ′
## 7
## ,e
## ′
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,e
## 
ifx=∇
(▸,FULL,e)otherwise ifς+SeS≥W
## X
(▸,ς+SeS,e
x)otherwise

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202552
## Function
## Identifier
Gas usage
## Mutations
## Ω
## M
## (ρ,φ,μ,(m,e))
machine= 8
g=10
let[p
## O
## ,p
## Z
## ,i]=φ
## 7⋅⋅⋅+3
letp=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
μ
p
## O
## ⋅⋅⋅+p
## Z
ifN
p
## O
## ⋅⋅⋅+p
## Z
## ⊆V
μ
## ∇otherwise
letn=min(n∈N,n~∈K(m))
letu=(v
## ▸
## ▸
## [0,0,...],a
## ▸
## ▸
## [∅,∅,...])
## 
ε
## ′
## ,φ
## ′
## 7
## ,m
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,m
## 
ifp=∇
(▸,HUH,m)otherwise if deblob(p)=∇
## (▸,n,m∪{ (n↦(p,u,i)) })otherwise
## Ω
## P
## (ρ,φ,μ,(m,e))
peek= 9
g=10
let[n,o,s,z]=φ
## 7⋅⋅⋅+4
## 
ε
## ′
## ,φ
## ′
## 7
## ,μ
## ′
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,μ
## 
ifN
o⋅⋅⋅+z
## ~⊆V
## ∗
## [
μ]
(▸,WHO,μ)otherwise ifn~∈K(m)
(▸,OOB,μ)otherwise ifN
s⋅⋅⋅+z
## ~⊆V
m[n]
u
(▸,OK,μ
## ′
## )otherwise
whereμ
## ′
## =μexceptμ
o⋅⋅⋅+z
## =(m[n]
u
## )
s⋅⋅⋅+z
## Ω
## O
## (ρ,φ,μ,(m,e))
poke= 10
g=10
let[n,s,o,z]=φ
## 7⋅⋅⋅+4
## 
ε
## ′
## ,φ
## ′
## 7
## ,m
## ′
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,m
## 
ifN
s⋅⋅⋅+z
## ~⊆V
## [
μ]
(▸,WHO,m)otherwise ifn~∈K(m)
(▸,OOB,m)otherwise ifN
o⋅⋅⋅+z
## ~⊆V
## ∗
m[n]
u
(▸,OK,m
## ′
## )otherwise
wherem
## ′
## =mexcept(m
## ′
## [n]
u
## )
o⋅⋅⋅+z
## =μ
s⋅⋅⋅+z
## Ω
## Z
## (ρ,φ,μ,(m,e))
pages= 11
g=10
let[n,p,c,r]=φ
## 7⋅⋅⋅+4
letu=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
m[n]
u
ifn∈K(m)
## ∇otherwise
letu
## ′
## =uexcept
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## (u
## ′
v
## )
pZ
## P
⋅⋅⋅+cZ
## P
## =
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## [0,0,...]ifr<3
## (u
v
## )
pZ
## P
⋅⋅⋅+cZ
## P
otherwise
## (u
## ′
a
## )
p⋅⋅⋅+c
## =
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## [∅,∅,...]ifr=0
[R,R,...]ifr=1∨r=3
[W,W,...]ifr=2∨r=4
## 
φ
## ′
## 7
## ,m
## ′
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## (
## WHO
## ,
m
## )
if
u
## =
## ∇
(HUH,m)otherwise ifr>4∨p<16∨p+c≥
## 2
## 32
## ~Z
## P
(HUH,m)otherwise ifr>2∧(u
a
## )
p⋅⋅⋅+c
## ∋∅
(OK,m
## ′
## )otherwise,wherem
## ′
## =mexceptm
## ′
## [n]
u
## =u
## ′

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202553
## Function
## Identifier
Gas usage
## Mutations
## Ω
## K
## (ρ,φ,μ,(m,e))
invoke= 12
g=10
let[n,o]=φ
## 7,8
let(g,w)=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
(g,w)∶E
## 8
(g)⌢E
## 8
## (w)=μ
o⋅⋅⋅+112
ifN
o⋅⋅⋅+112
## ⊆V
## ∗
μ
## (∇,∇)otherwise
let
## 
c,i
## ′
## ,g
## ′
## ,w
## ′
## ,u
## ′
## 
=Ψ(m[n]
p
## ,m[n]
i
## ,g,w,m[n]
u
## )
letμ
## ∗
## =μexceptμ
## ∗
o⋅⋅⋅+112
## =E
## 8
## 
g
## ′
## 
## ⌢E
## 8
## 
w
## ′
## 
letm
## ∗
## =mexcept
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
m
## ∗
## [n]
u
## =u
## ′
m
## ∗
## [n]
i
## =
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
i
## ′
## +skip(ı
## ′
## )+1ifc∈{
## ̵
h}×N
## R
i
## ′
otherwise
## 
ε
## ′
## ,φ
## ′
## 7
## ,φ
## ′
## 8
## ,μ
## ′
## ,m
## ′
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,φ
## 8
## ,μ,m
## 
ifg=∇
(▸,WHO,φ
## 8
,μ,m)otherwise ifn~∈m
(▸,HOST,h,μ
## ∗
## ,m
## ∗
)otherwise ifc=
## ̵
h×h
(▸,FAULT,x,μ
## ∗
## ,m
## ∗
)otherwise ifc=
## F
## ×x
(▸,OOG,φ
## 8
## ,μ
## ∗
## ,m
## ∗
)otherwise ifc=∞
(▸,PANIC,φ
## 8
## ,μ
## ∗
## ,m
## ∗
)otherwise ifc=☇
(▸,HALT,φ
## 8
## ,μ
## ∗
## ,m
## ∗
)otherwise ifc=∎
## Ω
## X
## (ρ,φ,μ,(m,e))
expunge= 13
g=10
letn=φ
## 7
## 
φ
## ′
## 7
## ,m
## ′
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
(WHO,m)ifn~∈K(m)
## 
m[n]
i
## ,m∖n
## 
otherwise
B.7.Accumulate Functions.This defines a number of functions broadly of the form(ρ
## ′
## ∈Z
## G
## ,φ
## ′
## ∈⟦N
## R
## ⟧
## 13
## ,μ
## ′
## ,(x
## ′
## ,y
## ′
## ))=
## Ω
## ◻
(ρ∈N
## G
,φ∈⟦N
## R
## ⟧
## 13
,μ∈M,(x,y)∈L
## 2
,...). Functions which have a result component which is equivalent to the cor-
responding argument may have said components elided in the description. Functions may also depend upon particular
additional parameters.
Other than the gas-counter which is explicitly defined, elements ofpvmstate are each assumed to remain unchanged
by the host-call unless explicitly specified.
ρ
## ′
≡ρ−g(B.19)
## 
ε
## ′
## ,φ
## ′
## ,μ
## ′
## ,x
## ′
## ,y
## ′
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## (∞,φ,μ,x,y)ifρ<g
(▸,φ,μ,x,y)except as indicated below otherwise
## (B.20)
## Function
## Identifier
Gas usage
## Mutations
## Ω
## B
## (ρ,φ,μ,(x,y))
bless= 14
g=10
let[m,a,v,r,o,n]=φ
## 7⋅⋅⋅+6
leta=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## E
## −1
## 4
## (μ
a⋅⋅⋅+4C
)ifN
a⋅⋅⋅+4C
## ⊆V
μ
## ∇otherwise
letz=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
{ (s↦g)whereE
## 4
(s)⌢E
## 8
## (g)=μ
o+12i⋅⋅⋅+12
Si∈N
n
}ifN
o⋅⋅⋅+12n
## ⊆V
μ
## ∇otherwise
## 
ε
## ′
## ,φ
## ′
## 7
## ,(x
## ′
e
## )
## (m,a,v,r,z)
## 
## =
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,(x
e
## )
## (m,a,v,r,z)
## 
if{z,a}∋∇
## 
▸,WHO,(x
e
## )
## (m,a,v,r,z)
## 
otherwise if(m,v,r) ~∈N
## 3
## S
## (▸,OK,
## ⎧
## ⎩
m,a,v,r,z
## ⎫
## ⎭
## )otherwise

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202554
## Function
## Identifier
Gas usage
## Mutations
## Ω
## A
## (ρ,φ,μ,(x,y))
assign= 15
g=10
let[c,o,a]=φ
## 7⋅⋅⋅+3
letq=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## [μ
o+32i⋅⋅⋅+32
Si<−N
## Q
]ifN
o⋅⋅⋅+32Q
## ⊆V
μ
## ∇otherwise
## 
ε
## ′
## ,φ
## ′
## 7
## ,(x
## ′
e
## )
q
## [c],(x
## ′
e
## )
a
## [c]
## 
## =
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,(x
e
## )
q
## [c],(x
e
## )
a
## [c]
## 
ifq=∇
(▸,CORE,(x
e
## )
q
## [c],(x
e
## )
a
[c])otherwise ifc≥C
(▸,HUH,(x
e
## )
q
## [c],(x
e
## )
a
[c])otherwise ifx
s
## ≠(x
e
## )
a
## [c]
(▸,WHO,(x
e
## )
q
## [c],(x
e
## )
a
[c])otherwise ifa~∈N
## S
(▸,OK,q,a)otherwise
## Ω
## D
## (ρ,φ,μ,(x,y))
designate= 16
g=10
leto=φ
## 7
letv=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## [μ
o+336i⋅⋅⋅+336
## Si<
## −N
## V
]ifN
o⋅⋅⋅+336V
## ⊆V
μ
## ∇otherwise
## 
ε
## ′
## ,φ
## ′
## 7
## ,(x
## ′
e
## )
i
## 
## =
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,(x
e
## )
i
## 
ifv=∇
(▸,HUH,(x
e
## )
i
)otherwise ifx
s
## ≠(x
e
## )
v
(▸,OK,v)otherwise
## Ω
## C
## (ρ,φ,μ,(x,y))
checkpoint= 17
g=10
y
## ′
## ≡x
φ
## ′
## 7
## ≡ρ
## ′
## Ω
## N
## (ρ,φ,μ,(x,y),t)
new= 18
g=10
let[o,l,g,m,f,i]=φ
## 7⋅⋅⋅+6
letc=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
μ
o⋅⋅⋅+32
ifN
o⋅⋅⋅+32
## ⊆V
μ
∧l∈N
## 2
## 32
## ∇otherwise
leta∈A∪{∇}=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## (c,s
## ▸
## ▸
## {},l
## ▸
## ▸
## { ((c,l)↦[]) },b
## ▸
## ▸
a
t
## ,g,m,p
## ▸
## ▸
## {},r
## ▸
## ▸
t,f,a
## ▸
## ▸
## 0,p
## ▸
## ▸
x
s
## )ifc≠∇
## ∇otherwise
lets=x
s
excepts
b
## =(x
s
## )
b
## −a
t
## 
ε
## ′
## ,φ
## ′
## 7
## ,x
## ′
i
## ,(x
## ′
e
## )
d
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,x
i
## ,(x
e
## )
d
## 
ifc=∇
## (
## ▸
## ,
## HUH
## ,
x
i
## ,(x
e
## )
d
)otherwise iff≠0∧x
s
## ≠(x
e
## )
m
(▸,CASH,x
i
## ,(x
e
## )
d
)otherwise ifs
b
## <(x
s
## )
t
(▸,FULL,x
i
## ,(x
e
## )
d
)otherwise ifx
s
## =(x
e
## )
r
∧i<S∧i∈K((x
e
## )
d
## )
## (▸,i,x
i
## ,(x
e
## )
d
∪d)otherwise ifx
s
## =(x
e
## )
r
∧i<S
whered={ (i↦a),(x
s
## ↦s) }
## (▸,x
i
## ,i
## ∗
## ,(x
e
## )
d
## ∪d)otherwise
wherei
## ∗
=check(S+(x
i
−S+42)mod(2
## 32
## −S−2
## 8
## ))
andd={ (x
i
## ↦a),(x
s
## ↦s) }
## Ω
## U
## (ρ,φ,μ,(x,y))
upgrade= 19
g=10
let[o,g,m]=φ
## 7⋅⋅⋅+3
letc=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
μ
o⋅⋅⋅+32
ifN
o⋅⋅⋅+32
## ⊆V
μ
## ∇otherwise
## 
ε
## ′
## ,φ
## ′
## 7
## ,
## (
x
## ′
s
## )
c
## ,(x
## ′
s
## )
g
## ,(x
## ′
s
## )
m
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,(x
s
## )
c
## ,(x
s
## )
g
## ,(x
s
## )
m
## 
ifc=∇
(▸,OK,c,g,m)otherwise

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202555
## Function
## Identifier
Gas usage
## Mutations
## Ω
## T
## (ρ,φ,μ,(x,y))
transfer= 20
g=10+t
let[d,a,l,o]=φ
## 7⋅⋅⋅+4
## ,
letd=(x
e
## )
d
lett∈X∪{∇}=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## (s
## ▸
## ▸
x
s
## ,d,a,m
## ▸
## ▸
μ
o⋅⋅⋅+W
## T
## ,g
## ▸
## ▸
l)ifN
o⋅⋅⋅+W
## T
## ⊆V
μ
## ∇otherwise
letb=(x
s
## )
b
## −a
let(c,t)=
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,0
## 
ift=∇
(WHO,0)otherwise ifd~∈K(d)
(LOW,0)otherwise ifl<d[d]
m
(CASH,0)otherwise ifb<(x
s
## )
t
(OK,l)otherwise
## 
ε
## ′
## ,φ
## ′
## 7
## ,
x
## ′
t
## ,
## (
x
## ′
s
## )
b
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,x
t
## ,(x
s
## )
b
## 
ifc=☇
## (▸,c,x
t
## ,(x
s
## )
b
)otherwise ifc≠OK
## (
## ▸
## ,
## OK
## ,
x
t
t,b)otherwise
## Ω
## J
## (ρ,φ,μ,(x,y),t)
eject= 21
g=10
let[d,o]=φ
## 7,8
leth=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
μ
o⋅⋅⋅+32
ifN
o⋅⋅⋅+32
## ⊆V
μ
## ∇otherwise
letd=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## (x
e
## )
d
## [d]ifd≠x
s
∧d∈K((x
e
## )
d
## )
## ∇otherwise
letl=max(81,d
o
## )−81
lets
## ′
## =x
s
excepts
## ′
b
## =(x
s
## )
b
## +d
b
## 
ε
## ′
## ,φ
## ′
## 7
## ,(x
## ′
e
## )
d
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,(x
e
## )
d
## 
ifh=∇
(▸,WHO,(x
e
## )
d
)otherwise ifd=∇ ∨d
c
## ≠E
## 32
## (x
s
## )
(▸,HUH,(x
e
## )
d
)otherwise ifd
i
## ≠2∨(h,l) ~∈d
l
(▸,OK,(x
e
## )
d
## ∖{d}∪{ (x
s
## ↦s
## ′
) })otherwise ifd
l
[h,l]=[x,y],y<t−D
(▸,HUH,(x
e
## )
d
## )otherwise
## Ω
## Q
## (ρ,φ,μ,(x,y))
query= 22
g=10
let[o,z]=φ
## 7,8
leth=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
μ
o⋅⋅⋅+32
ifN
o⋅⋅⋅+32
## ⊆V
μ
## ∇otherwise
leta=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## (x
s
## )
l
[h,z]if(h,z)∈K((x
s
## )
l
## )
## ∇otherwise
## 
ε
## ′
## ,φ
## ′
## 7
## ,φ
## ′
## 8
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,φ
## 8
## 
ifh=∇
(▸,NONE,0)otherwise ifa=∇
(▸,0,0)otherwise ifa=[]
## 
## ▸,1+2
## 32
x,0
## 
otherwise ifa=[x]
## 
## ▸,2+2
## 32
x,y
## 
otherwise ifa=[x,y]
## 
## ▸,3+2
## 32
x,y+2
## 32
z
## 
otherwise ifa=[x,y,z]

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202556
## Function
## Identifier
Gas usage
## Mutations
## Ω
## S
## (ρ,φ,μ,(x,y),t)
solicit= 23
g=10
let[o,z]=φ
## 7,8
leth=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
μ
o⋅⋅⋅+32
ifN
o⋅⋅⋅+32
## ⊆V
μ
## ∇otherwise
leta=
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
x
s
except:
a
l
[(h,z)]=[]ifh≠∇ ∧(h,z) ~∈K((x
s
## )
l
## )
a
l
## [(h,z)]=(x
s
## )
l
## [(h,z)]
tif(x
s
## )
l
## [(h,z)]=[x,y]
## ∇otherwise
## 
ε
## ′
## ,φ
## ′
## 7
## ,x
## ′
s
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,x
s
## 
ifh=∇
(▸,HUH,x
s
)otherwise ifa=∇
(▸,FULL,x
s
)otherwise ifa
b
## <a
t
(▸,OK,a)otherwise
## Ω
## F
## (ρ,φ,μ,(x,y),t)
forget= 24
g=10
let[o,z]=φ
## 7,8
leth=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
μ
o⋅⋅⋅+32
ifN
o⋅⋅⋅+32
## ⊆V
μ
## ∇otherwise
leta=
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
x
s
except:
## K(a
l
)=K((x
s
## )
l
## )∖{ (h,z) },
## K(a
p
)=K((x
s
## )
p
## )∖{h}
## ¡if(x
s
## )
l
[h,z]∈{ [],[x,y] }, y<t−D
a
l
## [h,z]=[x,t]if(x
s
## )
l
## [h,z]=[x]
a
l
## [h,z]=[w,t]if(x
s
## )
l
[h,z]=[x,y,w], y<t−D
## ∇otherwise
## 
ε
## ′
## ,φ
## ′
## 7
## ,x
## ′
s
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,x
s
## 
ifh=∇
(▸,HUH,x
s
)otherwise ifa=∇
(▸,OK,a)otherwise
## Ω
## Q
## (ρ,φ,μ,(x,y))
yield= 25
g=10
leto=φ
## 7
leth=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
μ
o⋅⋅⋅+32
ifN
o⋅⋅⋅+32
## ⊆V
μ
## ∇otherwise
## 
ε
## ′
## ,φ
## ′
## 7
## ,x
## ′
y
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,x
y
## 
ifh=∇
(▸,OK,h)otherwise
## Ω
## P
## (ρ,φ,μ,(x,y))
provide= 26
g=10
let[o,z]=φ
## 8,9
letd=(x
e
## )
d
lets=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
x
s
ifφ
## 7
## =2
## 64
## −1
φ
## 7
otherwise
leti=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
μ
o⋅⋅⋅+z
ifN
o⋅⋅⋅+z
## ⊆V
μ
## ∇otherwise
leta=
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
d[s]ifs∈K(d)
## ∅otherwise
## 
ε
## ′
## ,φ
## ′
## 7
## ,x
## ′
p
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## ☇,φ
## 7
## ,x
p
## 
ifi=∇
(▸,WHO,x
p
)otherwise ifa=∅
(▸,HUH,x
p
)otherwise ifa
l
[(H(i),z)]≠[]
(▸,HUH,x
p
)otherwise if(s,i)∈x
p
(▸,OK,x
p
## ∪{ (s,i) })otherwise

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202557
AppendixC.Serialization Codec
C.1.Common Terms.Our codec functionEis used to serialize some term into a sequence of octets. We define the
deserialization functionE
## −1
as the inverse ofEand able to decode some sequence into the original value. The codec is
designed such that exactly one value is encoded into any given sequence of octets, and in cases where this is not desirable
then we use special codec functions.
C.1.1.Trivial Encodings.We define the serialization of∅as the empty sequence:
## (C.1)E(∅)≡[]
We also define the serialization of an octet-sequence as itself:
(C.2)E(x∈B)≡x
We define anonymous tuples to be encoded as the concatenation of their encoded elements:
(C.3)E((a,b,...))≡E(a)⌢E(b)⌢...
Passing multiple arguments to the serialization functions is equivalent to passing a tuple of those arguments. Formally:
E(a,b,...)≡E((a,b,...))
## (C.4)
We define general natural number serialization, able to encode naturals of up to2
## 64
, as:
## (C.5)E∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## N
## 2
## 64
## →B
## 1∶9
x↦
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## [0]ifx=0
## 
## 2
## 8
## −2
## 8−l
## +

x
## 2
## 8l
## 
## ⌢E
l
## 
xmod 2
## 8l
## 
if∃l∈N
## 8
## ∶2
## 7l
## ≤x<2
## 7(l+1)
## 
## 2
## 8
## −1
## 
## ⌢E
## 8
(x)otherwise ifx<2
## 64
C.1.2.Sequence Encoding.We define the sequence serialization functionE
## 
## ⟦T⟧
## 
for anyTwhich is itself a subset of the
domain ofE. We simply concatenate the serializations of each element in the sequence in turn:
(C.6)E([i
## 0
## ,i
## 1
,...])≡E(i
## 0
)⌢E(i
## 1
## )⌢...
Thus, conveniently, fixed length octet sequences (e.g. hashesHand its variants) have an identity serialization.
C.1.3.Discriminator Encoding.When we have sets of heterogeneous items such as a union of different kinds of tuples
or sequences of different length, we require a discriminator to determine the nature of the encoded item for successful
deserialization. Discriminators are encoded as a natural and are encoded immediately prior to the item.
We generally use alength discriminatorwhen serializing sequence terms which have variable length (e.g. general blobs
Bor unbound numeric sequences⟦N⟧) (though this is omitted in the case of fixed-length terms such as hashesH).
## 19
## In
this case, we simply prefix the term its length prior to encoding. Thus, for some termy∈(x∈B,...), we would generally
define its serialized form to beE(SxS)⌢E(x)⌢.... To avoid repetition of the term in such cases, we define the notation
↕xto mean that the term of valuexis variable in size and requires a length discriminator. Formally:
(C.7)↕x≡(SxS,x)thusE(↕x)≡E(SxS)⌢E(x)
We also define a convenient discriminator operator ¿xspecifically for terms defined by some serializable set in union
with∅(generally denoted for some setSasS?):
## ¿x≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## 0ifx=∅
## (1,x)otherwise
## (C.8)
C.1.4.Bit Sequence Encoding.A sequence of bitsb∈bis a special case since encoding each individual bit as an octet
would be very wasteful. We instead pack the bits into octets in order of least significant to most, and arrange into an
octet stream. In the case of a variable length sequence, then the length is prefixed as in the general case.
## E(b∈b)≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## []ifb=[]
## 
i<min(8,SbS)
## ∑
i=0
b
i
## ⋅2
i
⌢E(b
## 8...
## )otherwise
## (C.9)
C.1.5.Dictionary Encoding.In general, dictionaries are placed in the Merkle trie directly (see appendixEfor details).
However, small dictionaries may reasonably be encoded as a sequence of pairs ordered by the key. Formally:
(C.10)∀K,V∶E(d∈jK→Vo)≡E(↕[(E(k),E(d[k])) Sk∈K(d)
## ^
## ^
k])
C.1.6.Set Encoding.For any values which are sets and don’t already have a defined encoding above, we define the
serialization of a set as the serialization of the set’s elements in proper order. Formally:
(C.11)E({a,b,c,...})≡E(a)⌢E(b)⌢E(c)⌢...wherea<b<c<...
## 19
Note that since specific values may belong to both sets which would need a discriminator and those that would not then we are sadly
unable to introduce a function capable of serializing corresponding to theterm’s limitation. A more sophisticated formalism than basic
set-theory would be needed, capable of taking into account not simply the value but the term from which or to which it belongs in order
to do this succinctly.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202558
C.1.7.Fixed-length Integer Encoding.We first define the trivial natural number serialization functions which are sub-
scripted by the number of octets of the final sequence. Values are encoded in a regular little-endian fashion. This is
utilized for almost all integer encoding across the protocol. Formally:
## (C.12)E
l∈N
## ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## N
## 2
## 8l
## →B
l
x↦
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## []ifl=0
[xmod 256]⌢E
l−1
## 
x
## 256
## 
otherwise
For non-natural arguments,E
l∈N
corresponds to the definitions ofE, except that recursive elements are made asE
l
rather thanE. Thus:
## E
l∈N
(a,b,...)≡E
l
((a,b,...))(C.13)
## E
l∈N
((a,b,...))≡E
l
(a)⌢E
l
(b)⌢...(C.14)
## E
l
## ∈
## N
## ([i
## 0
## ,i
## 1
## ,...])≡E
l
## (i
## 0
## )⌢E
l
## (i
## 1
## )⌢...
## (C.15)
And so on.
C.2.Block Serialization.A blockBis serialized as a tuple of its elements in regular order, as implied in equations
4.2,4.3and5.1. For the header, we define both the regular serialization and the unsigned serializationE
## U
## . Formally:
## E(B)=E(H,E
## T
## (E
## T
## ),E
## P
## (E
## P
## ),E
## G
## (E
## G
## ),E
## A
## (E
## A
## ),E
## D
## (E
## D
## ))(C.16)
## E
## T
## (E
## T
## )=E(↕E
## T
## )(C.17)
## E
## P
## (E
## P
## )=E(↕[(E
## 4
(s),↕d) S (s,d)<−E
## P
## ])(C.18)
## E
## G
## (E
## G
)=E(↕[(r,E
## 4
(t),↕[(E
## 2
## (v),s) S (v,s)<
−a]) S (r,t,a)<−E
## G
## ])(C.19)
## E
## A
## (
## E
## A
## )
## =E
## (
## ↕
## [(
a,f,E
## 2
## (
v
## )
## ,s
## ) S (
a,f,v,s
## )
## <
## −E
## A
## ])
## (C.20)
## E
## D
((v,c,f))=E(↕[(r,E
## 4
(a),[(v,E
## 2
## (i),s) S (v,i,s)<
−j]) S (r,a,j)<−v],↕c,↕f)(C.21)
## E(H)=E(E
## U
## (H),H
## S
## )(C.22)
## E
## U
## (H)=E(H
## P
## ,H
## R
## ,H
## X
## ,E
## 4
## (H
## T
## ),¿H
## E
## ,¿H
## W
## ,E
## 2
## (H
## I
## ),H
## V
## ,↕H
## O
## )(C.23)
E(x∈C)≡E(x
a
## ,x
s
## ,x
b
## ,x
l
## ,E
## 4
## (x
t
## ),↕x
p
## )
## (C.24)
E(x∈Y)≡E(x
p
## ,E
## 4
## (x
l
## ),x
u
## ,x
e
## ,E
## 2
## (x
n
## ))(C.25)
E(d∈D)≡E(E
## 4
## (d
s
## ),d
c
## ,d
y
## ,E
## 8
## (d
g
),O(d
l
## ),d
u
## ,d
i
## ,d
x
## ,d
z
## ,d
e
## )(C.26)
E(r∈R)≡E(r
s
## ,r
c
## ,r
c
## ,r
a
## ,r
g
## ,↕r
t
## ,↕r
l
## ,↕r
d
## )(C.27)
E(p∈P)≡E(E
## 4
## (p
h
## ),p
u
## ,p
c
## ,↕p
j
## ,↕p
f
## ,↕p
w
## )(C.28)
E(w∈W)≡EE
## 4
## (w
s
## ),w
c
## ,E
## 8
## (w
g
## ),E
## 8
## (w
a
## ),E
## 2
## (w
e
## ),↕w
y
## ,
## Õ
## ×
## Ö
## I
## #
## (w
i
),↕[(h,E
## 4
## (i)) S (h,i)<−w
x
## ](C.29)
E(x∈T)≡E(x
y
## ,x
e
## )
## (C.30)
## E
## X
(x∈X)≡E(E
## 4
## (x
s
## ),E
## 4
## (x
d
## ),E
## 8
## (x
a
## ),x
m
## ,E
## 8
## (x
g
## ))
## (C.31)
## E
## U
(x∈U)≡E(x
p
## ,x
e
## ,x
a
## ,x
y
## ,x
g
,O(x
l
## ),↕x
t
## )(C.32)
E(x∈I)≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## E(0,E
## U
(o))ifx∈U
## E(1,E
## X
(o))ifx∈X
## (C.33)
O(o∈E∪B)≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
(0,↕o)ifo∈B
## 1ifo=∞
## 2ifo=☇
## 3ifo=⊚
## 4ifo=⊖
5ifo=BAD
6ifo=BIG
## (C.34)
## I
## 
h∈H∪H
## ⊞
,i∈N
## 2
## 15
## 
## ≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
(h,E
## 2
(i))ifh∈H
## 
r,E
## 2
## 
i+2
## 15
## 
if∃r∈H,h=r
## ⊞
## (C.35)
Note the use ofOabove to succinctly encode the result of a work item and the slight transformations ofE
## G
and
## E
## P
to take account of the fact their inner tuples contain variable-length sequence termsaandpwhich need length
discriminators.
AppendixD.State Merklization
The Merklization process defines a cryptographic commitment from which arbitrary information within state may be
provided as being authentic in a concise and swift fashion. We describe this in two stages; the first defines a mapping

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202559
from 31-octet sequences to (unlimited) octet sequences in a process calledstate serialization. The second forms a 32-octet
commitment from this mapping in a process calledMerklization.
D.1.Serialization.The serialization of state primarily involves placing all the various components ofσinto a single
mapping from 31-octet sequencestate-keysto octet sequences of indefinite length. The state-key is constructed from a
hash component and a chapter component, equivalent to either the index of a state component or, in the case of the
inner dictionaries ofδ, a service index.
We define the state-key constructor functionsCas:
## (D.1)C∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## N
## 2
## 8
## ∪(N
## 2
## 8
## ,N
## S
## )∪(N
## S
## ,B)→B
## 31
i∈N
## 2
## 8
## ↦[i,0,0,...]
(i,s∈N
## S
## )↦[i,n
## 0
## ,0,n
## 1
## ,0,n
## 2
## ,0,n
## 3
,0,0,...]wheren=E
## 4
## (s)
## (
s,h
## )
## ↦
## [
n
## 0
## ,a
## 0
## ,n
## 1
## ,a
## 1
## ,n
## 2
## ,a
## 2
## ,n
## 3
## ,a
## 3
## ,a
## 4
## ,a
## 5
## ,...,a
## 26
]wheren=E
## 4
(s),a=H(h)
The state serialization is then defined as the dictionary built from the amalgamation of each of the components.
Cryptographic hashing ensures that there will be no duplicate state-keys given that there are no duplicate inputs toC.
Formally, we defineTwhich transforms some stateσinto its serialized form:
## (D.2)
## T(σ)≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
C(1)↦E([↕xSx<
## −α]),
C(2)↦E(φ),
## C
## (
## 3
## )
## ↦
## E(
## ↕
## [(
h,b,s,
## ↕
p
## ) S (
h,b,s,
p)
## <−β
## H
## ]
## ,E
## M
## (
β
## B
## ))
## ,
## C(4)↦E
## ⎛
## ⎝
γ
## P
## ,γ
## Z
## ,
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## 0ifγ
## S
## ∈⟦T⟧
## E
## 1ifγ
## S
## ∈D
## ∽
## HI
## E
## ⎫
## ⎪
## ⎪
## ⎬
## ⎪
## ⎪
## ⎭
## ,γ
## S
## ,↕γ
## A
## ⎞
## ⎠
## ,
C(5)↦E(↕[x∈ψ
## G
## ^
## ^
x],↕[x∈ψ
## B
## ^
## ^
x],↕[x∈ψ
## W
## ^
## ^
x],↕[x∈ψ
## O
## ^
## ^
x]),
C(6)↦E(η),
C(7)↦E(ι),
C(8)↦E(κ),
C(9)↦E(λ),
C(10)↦E([¿(r,E
## 4
## (t)) S (r,t)<
## −ρ]),
## C(11)↦E
## 4
## (τ),
## C(12)↦E(E
## 4
## (χ
## M
## ,χ
## A
## ,χ
## V
## ,χ
## R
## ),χ
## Z
## ),
## C(13)↦E(E
## 4
## (π
## V
## ,π
## L
## ),π
## C
## ,π
## S
## ),
## C(14)↦E
## ([
## ↕
## [(
r,↕d
## ) S (
r,d
## )
## <
## −i] Si<−ω]),
C(15)↦E([↕iSi<−ξ]),
## C(16)↦E(↕[(E
## 4
(s),E(h)) S (s,h)<
## −θ]),
∀(s↦a)∈δ∶C(255,s)↦E(0,a
c
## ,E
## 8
## (a
b
## ,a
g
## ,a
m
## ,a
o
## ,a
f
## ),E
## 4
## (a
i
## ,a
r
## ,a
a
## ,a
p
## )),
## ∀(s↦a)∈δ,(k↦v)∈a
s
∶C(s,E
## 4
## 
## 2
## 32
## −1
## 
## ⌢k)↦v,
## ∀(s↦a)∈δ,(h↦p)∈a
p
∶C(s,E
## 4
## 
## 2
## 32
## −2
## 
## ⌢h)↦p,
## ∀(s↦a)∈δ,((h,l)↦t)∈a
l
∶C(s,E
## 4
(l)⌢h)↦E(↕[E
## 4
## (x) Sx<−t])
Note that most rows describe a single mapping between a key derived from a natural and the serialization of a state
component. However, the final four rows each define sets of mappings since these items act over all service accounts and
in the case of the final three rows, the keys of a nested dictionary with the service.
Also note that all non-discriminator numeric serialization in state is done in fixed-length according to the size of the
term.
Finally, be aware that
## J
amdoes not allow service storage keys to be directly inspected or enumerated. Thus the
key values themselves are not required to be known by implementations, and only the Merklisation-ready serialisation
is important, which is a fixed-size hash (alongside the service index and item marker). Implementations are free to use
this fact in order to avoid storing the keys themselves.
D.2.Merklization.WithTdefined, we now define the rest ofM
σ
which primarily involves transforming the serialized
mapping into a cryptographic commitment. We define this commitment as the root of the binary Patricia Merkle Trie
with a format optimized for modern compute hardware, primarily by optimizing sizes to fit succinctly into typical memory
layouts and reducing the need for unpredictable branching.
D.2.1.Node Encoding and Trie Identification.We identify (sub-)tries as the hash of their root node, with one exception:
empty (sub-)tries are identified as the zero-hash,H
## 0
## .
Nodes are fixed in size at 512 bit (64 bytes). Each node is either a branch or a leaf. The first bit discriminate between
these two types.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202560
In the case of a branch, the remaining 511 bits are split between the two child node hashes, using the last 255 bits of
the 0-bit (left) sub-trie identity and the full 256 bits of the 1-bit (right) sub-trie identity.
Leaf nodes are further subdivided into embedded-value leaves and regular leaves. The second bit of the node discrim-
inates between these.
In the case of an embedded-value leaf, the remaining 6 bits of the first byte are used to store the embedded value size.
The following 31 bytes are dedicated to the state key. The last 32 bytes are defined as the value, filling with zeroes if its
length is less than 32 bytes.
In the case of a regular leaf, the remaining 6 bits of the first byte are zeroed. The following 31 bytes store the state
key. The last 32 bytes store the hash of the value.
Formally, we define the encoding functionsBandL:
## B∶
## ⎧
## ⎩
## H,H
## ⎫
## ⎭
## →b
## 512
## (l,r)↦[0]⌢bits(l)
## 1...
## ⌢bits(r)
## (D.3)
## L∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## B
## 31
## ,B
## ⎫
## ⎭
## →b
## 512
## (k,v)↦
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
[1,0]⌢bits(E
## 1
(SvS))
## 2...
⌢bits(k)⌢bits(v)⌢[0,0,...]ifSvS≤32
[1,1,0,0,0,0,0,0]⌢bits(k)⌢bits(H(v))otherwise
## (D.4)
We may then define the basic Merklization functionM
σ
as:
## M
σ
(σ)≡M({ (bits(k)↦(k,v)) S (k↦v)∈T(σ) })(D.5)
## M(d∶jb→
## ⎧
## ⎩
## B
## 31
## ,B
## ⎫
## ⎭
o)≡
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## H
## 0
ifSdS=0
## H
## 
bits
## −1
(L(k,v))
## 
ifV(d)={ (k,v) }
## H
## 
bits
## −1
(B(M(l),M(r)))
## 
otherwise
where∀b,p∶(b↦p)∈d⇔(b
## 1...
## ↦p)∈
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
lifb
## 0
## =0
rifb
## 0
## =1
## (D.6)
AppendixE.General Merklization
E.1.Binary Merkle Trees.The Merkle tree is a cryptographic data structure yielding a hash commitment to a specific
sequence of values. It providesO(N)computation andO(log(N))proof size for inclusion. Thiswell-balancedformulation
ensures that the maximum depth of any leaf is minimal and that the number of leaves at that depth is also minimal.
The underlying function for our Merkle trees is thenodefunctionN, which accepts some sequence of blobs of some
lengthnand provides either such a blob back or a hash:
## (E.1)N∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## ⟦B
n
## ⟧,B→H
## ⎫
## ⎭
## →B
n
## ∪H
(v,H)↦
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## H
## 0
ifSvS=0
v
## 0
ifSvS=1
H($node⌢N(v
## ...⌈
SvS
## ~2⌉
,H)⌢N(v
## ⌈
SvS
## ~2⌉...
,H))otherwise
The astute reader will realize that if ourB
n
happens to be equivalentHthen this function will always evaluate intoH.
That said, for it to be secure care must be taken to ensure there is no possibility of preimage collision. For this purpose
we include the hash prefix$nodeto minimize the chance of this; simply ensure any items are hashed with a different
prefix and the system can be considered secure.
We also define thetracefunctionT, which returns each opposite node from top to bottom as the tree is navigated to
arrive at some leaf corresponding to the item of a given index into the sequence. It is useful in creating justifications of
data inclusion.
## (E.2)
## T∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## ⟦B
n
## ⟧,N
SvS
## ,B→H
## ⎫
## ⎭
## →⟦B
n
## ∪H⟧
## (
v,i,H
## )
## ↦
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## 
## N(P
## 
(v,i),H)
## 
## ⌢T(P
## ⊺
(v,i),i−P
## I
(v,i),H)if
## S
v
## S
## >1
## []otherwise
whereP
s
## (v,i)≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
v
## ...⌈
SvS
## ~2⌉
if(i<⌈
SvS
## ~2⌉)=s
v
## ⌈
SvS
## ~2⌉...
otherwise
andP
## I
## (v,i)≡
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## 0ifi<⌈
SvS
## ~2⌉
## ⌈
SvS
## ~2⌉otherwise
From this we define our other Merklization functions.
E.1.1.Well-Balanced Tree.We define the well-balanced binary Merkle function asM
## B
## :
## (E.3)
## M
## B
## ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## ⟦B⟧,B→H
## ⎫
## ⎭
## →H
(v,H)↦
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## H(v
## 0
)ifSvS=1
N(v,H)otherwise

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202561
This is suitable for creating proofs on data which is not much greater than 32 octets in length since it avoids hashing
each item in the sequence. For sequences with larger data items, it is better to hash them beforehand to ensure proof-size
is minimal since each proof will generally contain a data item.
Note: In the case that no hash function argumentHis supplied, we may assume Blake 2b.
E.1.2.Constant-Depth Tree.We define the constant-depth binary Merkle function asM. We define two corresponding
functions for working with subtree pages,J
x
andL
x
. The latter provides a single page of leaves, themselves hashed,
prefixed data. The former provides the Merkle path to a single page. Both assume size-aligned pages of size2
x
and
accept page indices.
## M∶
## ⎧
## ⎩
## ⟦B⟧,B→H
## ⎫
## ⎭
## →H
(v,H)↦N(C(v,H),H)
## (E.4)
## J
x
## ∶
## ⎧
## ⎩
## ⟦B⟧,N
SvS
## ,B→H
## ⎫
## ⎭
## →⟦H⟧
(v,i,H)↦T(C(v,H),2
x
i,H)
## ...max(0,⌈log
## 2
(max(1,SvS))−x⌉)
## (E.5)
## L
x
## ∶
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## ⟦B⟧,N
## Sv
## S
## ,B→H
## ⎫
## ⎭
## →⟦H⟧
(v,i,H)↦
## 
## H($leaf⌢l)
## T
l<−v
## 2
x
i...min(2
x
i+2
x
,SvS)
## 
## (E.6)
For the latter justificationJ
x
to be acceptable, we must assume the target observer also knows not merely the value
of the item at the given index, but also all other leaves within its2
x
size subtree, given byL
x
## .
As above, we may assume a default value forHof Blake 2b.
For justifications and Merkle root calculations, a constancy preprocessor functionCis applied which hashes all data
items with a fixed prefix “leaf” and then pads the overall size to the next power of two with the zero hashH
## 0
## :
## (E.7)C∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## ⟦B⟧,B→H
## ⎫
## ⎭
## →⟦H⟧
(v,H)↦v
## ′
where
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## T
v
## ′
## T
## =2
## ⌈log
## 2
(max(1,SvS))⌉
v
## ′
i
## =
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## H($leaf⌢v
i
)ifi<SvS
## H
## 0
otherwise
E.2.Merkle Mountain Ranges and Belts.The Merkle Mountain Range (mmr) is an append-only cryptographic
data structure which yields a commitment to a sequence of values. Appending to anmmrand proof of inclusion of some
item within it are bothO(log(N))in time and space for the size of the set.
We define a Merkle Mountain Range as being within the set⟦H?⟧, a sequence of peaks, each peak the root of a
Merkle tree containing2
i
items whereiis the index in the sequence. Since we support set sizes which are not always
powers-of-two-minus-one, some peaks may be empty,∅rather than a Merkle root.
Since the sequence of hashes is somewhat unwieldy as a commitment, Merkle Mountain Ranges are themselves
generally hashed before being published. Hashing them removes the possibility of further appending so the range itself
is kept on the system which needs to generate future proofs.
We define themmbappend functionAas:
## (E.8)
## A∶
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## ⟦H?⟧,H,B→H
## ⎫
## ⎭
## →⟦H?⟧
(r,l,H)↦P(r,l,0,H)
whereP∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## ⟦H?⟧,H,N,B→H
## ⎫
## ⎭
## →⟦H?⟧
(r,l,n,H)↦
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
r
lifn≥SrS
R(r,n,l)ifn<SrS∧r
n
## =∅
P(R(r,n,∅),H(r
n
⌢l),n+1,H)otherwise
andR∶
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## ⟦T⟧,N,T
## ⎫
## ⎭
## →⟦T⟧
## (s,i,v)↦s
## ′
wheres
## ′
## =sexcepts
## ′
i
## =v
We define themmrencoding function asE
## M
## :
## (E.9)
## E
## M
## ∶
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## ⟦H?⟧→B
b↦E(↕[¿xSx<
## −b])
We define themmrsuper-peak function asM
## R
## :
## (E.10)M
## R
## ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⟦H?⟧→H
b↦
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## H
## 0
ifShS=0
h
## 0
ifShS=1
## H
## K
## 
$peak⌢M
## R
## 
h
...ShS−1
## 
## ⌢h
ShS−1
## 
otherwise
whereh=[hSh<
## −b,h≠∅]

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202562
AppendixF.Shuffling
The Fisher-Yates shuffle function is defined formally as:
## (F.1)
∀T,l∈N∶F∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## ⎧
## ⎩
## ⟦T⟧
l
## ,⟦N⟧
l∶
## ⎫
## ⎭
## →⟦T⟧
l
## (s,r)↦
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## [s
r
## 0
modl
]⌢F(s
## ′
## ...l−1
## ,r
## 1...
## )wheres
## ′
## =sexcepts
## ′
r
## 0
modl
## =s
l−1
ifs≠[]
## []otherwise
Since it is often useful to shuffle a sequence based on some random seed in the form of a hash, we provide a secondary
form of the shuffle functionFwhich accepts a 32-byte hash instead of the numeric sequence. We defineQ, the numeric-
sequence-from-hash function, thus:
∀l∈N∶Q
l
## ∶
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## H→⟦N
## 2
## 32
## ⟧
l
h↦
## 
## E
## −1
## 4
## 
H(h⌢E
## 4
## (⌊
i
## ~8⌋))
## 4imod 32⋅⋅⋅+4
##  T
i<−N
l
## 
## (F.2)
∀T,l∈N∶F∶
## ⎧
## ⎩
## ⟦T⟧
l
## ,H
## ⎫
## ⎭
## →⟦T⟧
l
(s,h)↦F(s,Q
l
## (h))
## (F.3)
AppendixG.Bandersnatch VRF
The Bandersnatch curve is defined by Masson, Sanso, and Zhang2021.
The singly-contextualized Bandersnatch Schnorr-like signatures
## ∽
## V
m
k
⟨c⟩are defined as a formulation under theIETF
vrftemplate specified by Hosseini and Galassi2024(as IETF VRF) and further detailed by Goldberg et al.2023.
## ∽
## V
m∈B
k∈
## ∽
## H
## ⟨
c∈H
## ⟩
## ⊂B
## 96
## ≡
## {
x
## S
x∈B
## 96
## ,verify(k,c,m,x)=⊺
## }
## (G.1)
## Ys∈
## ∽
## V
m
k
⟨c⟩∈H≡output(xSx∈
## ∽
## V
m
k
## ⟨c⟩)
## ...32
## (G.2)
The singly-contextualized Bandersnatch Ringvrfproofs
## ○
## V
m
r
⟨c⟩are a zk-snark-enabled analogue utilizing the Pedersen
vrf, also defined by Hosseini and Galassi2024and further detailed by Jeffrey Burdges et al.2023.
## OD
## ∽
## HI∈
## ○
B≡commit(D
## ∽
## HI)(G.3)
## ○
## V
m∈B
r∈
## ○
## B
⟨c∈H⟩⊂B
## 784
≡{xSx∈B
## 784
,verify(r,c,m,x)=⊺}(G.4)
## Yp∈
## ○
## V
m
r
⟨c⟩∈H≡output(xSx∈
## ○
## V
m
r
## ⟨c⟩)
## ...32
## (G.5)
Note that in the case a key
## ∽
Hhas no corresponding Bandersnatch point when constructing the ring, then the Ban-
dersnatchpadding pointas stated by Hosseini and Galassi2024should be substituted.
AppendixH.Erasure Coding
The foundation of the data-availability and distribution system of
## J
amis a systematic Reed-Solomon erasure coding
function ingf(2
## 16
) of rate 342:1023, the same transform as done by the algorithm of Lin, Chung, and Han2014. We use
a little-endianB
## 2
form of the 16-bitgfpoints with a functional equivalence given byE
## 2
. From this we may assume the
encoding functionC∶⟦B
## 2
## ⟧
## 342
## →⟦B
## 2
## ⟧
## 1023
and the recovery functionR∶{[
## ⎧
## ⎩
## B
## 2
## ,N
## 1023
## ⎫
## ⎭
## ]}
## 342
## →⟦B
## 2
## ⟧
## 342
. Encoding is done
by extrapolating a data blob of size 684 octets (provided inChere as 342 octet pairs) into 1,023 octet pairs. Recovery
is done by collecting together any distinct 342 octet pairs, together with their indices, and transforming this into the
original sequence of 342 octet pairs.
Practically speaking, this allows for the eﬀicient encoding and recovery of data whose size is a multiple of 684 octets.
Data whose length is not divisible by 684 must be padded (we pad with zeroes). We use this erasure-coding in two
contexts within the
## J
amprotocol; one where we encode variable sized (but typically very large) data blobs for the Audit
daand block-distribution system, and the other where we encode much smaller fixed-size datasegmentsfor the Import
dasystem.
For the Importdasystem, we deal with an input size of 4,104 octets resulting in data-parallelism of order six. We
may attain a greater degree of data parallelism if encoding or recovering more than one segment at a time though for
recovery, we may be restricted to requiring each segment to be formed from the same set of indices (depending on the
specific algorithm).
H.1.Blob Encoding and Recovery.We assume some data blobd∈B
## 684k
,k∈N. This blob is split into a whole
number ofkpieces, each a sequence of 342 octet pairs. Each piece is erasure-coded usingCas above to give 1,023 octet
pairs per piece.
The resulting matrix is grouped by its pair-index and concatenated to form 1,023chunks, each ofkoctet-pairs. Any
342 of these chunks may then be used to reconstruct the original datad.
Formally we begin by defining two utility functions for splitting some large sequence into a number of equal-sized
sub-sequences and for reconstituting such subsequences back into a single large sequence:
∀n∈N,k∈N∶split
n
(d∈B
kn
## )∈⟦B
n
## ⟧
k
## ≡
## 
d
## 0⋅⋅⋅+n
## ,d
n⋅⋅⋅+n
## ,⋯,d
## (k−1)n⋅⋅⋅+n
## 
## (H.1)

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202563
∀n∈N,k∈N∶join(c∈⟦B
n
## ⟧
k
## )∈B
kn
## ≡c
## 0
## ⌢c
## 1
## ⌢...(H.2)
We define the transposition operator hence:
## (H.3)
## T
## [[x
## 0,0
## ,x
## 0,1
## ,x
## 0,2
## ,...],[x
## 1,0
## ,x
## 1,1
## ,...],...]≡[[x
## 0,0
## ,x
## 1,0
## ,x
## 2,0
## ,...],[x
## 0,1
## ,x
## 1,1
## ,...],...]
We may then define our erasure-code chunking function which accepts an arbitrary sized data blob whose length
divides wholly into 684 octets and results in a sequence of 1,023 smaller blobs:
## (H.4)C
k∈N
## ∶
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
## B
## 684k
## →⟦B
## 2k
## ⟧
## 1023
d↦join
## #
## (
## T
C(p) Up<−
## T
split
## #
## 2
## (split
## 2k
## (d)))
The original data may be reconstructed with any 342 of the 1,023 resultant items (along with their indices). If the
original 342 items are known then reconstruction is just their concatenation.
## (H.5)R
k∈N
## ∶
## ⎧
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎪
## ⎪
## ⎪
## ⎩
## {[
## ⎧
## ⎩
## B
## 2k
## ,N
## 1023
## ⎫
## ⎭
## ]}
## 342
## →B
## 684k
c↦
## ⎧
## ⎪
## ⎪
## ⎨
## ⎪
## ⎪
## ⎩
E([xS (x,i)<
## −[(x,i)∈c
## ^
## ^
i]])if{iS (x,i)∈c}=N
## 342
join(join
## #
## (
## T
[R({ (split
## 2
## (x)
p
,i) S (x,i)∈c}) Sp∈N
k
## ]))always
Segment encoding/decoding may be done using the same functions albeit with a constantk=6.
H.2.Code Word representation.For the sake of brevity we call each octet pair aword. The code words (including
the message words) are treated as element ofF
## 2
## 16
finite field. The field is generated as an extension ofF
## 2
using the
irreducible polynomial:
(H.6)x
## 16
## +x
## 5
## +x
## 3
## +x
## 2
## +1
## Hence:
## (H.7)F
## 2
## 16
## ≡
## F
## 2
## [x]
x
## 16
## +x
## 5
## +x
## 3
## +x
## 2
## +1
We name the generator of
## F
## 2
## 16
## F
## 2
, the root of the above polynomial,αas such:F
## 2
## 16
## =F
## 2
## (α).
Instead of using the standard basis
## 
## 1,α,α
## 2
## ,...,α
## 15
## 
, we opt for a representation ofF
## 2
## 16
which performs more
eﬀiciently for the encoding and the decoding process. To that aim, we name this specific representation ofF
## 2
## 16
as
## ̃
## F
## 2
## 16
and define it as a vector space generated by the following Cantor basis:
v
## 0
## 1
v
## 1
α
## 15
## +α
## 13
## +α
## 11
## +α
## 10
## +α
## 7
## +α
## 6
## +α
## 3
## +α
v
## 2
α
## 13
## +α
## 12
## +α
## 11
## +α
## 10
## +α
## 3
## +α
## 2
## +α
v
## 3
α
## 12
## +α
## 10
## +α
## 9
## +α
## 5
## +α
## 4
## +α
## 3
## +α
## 2
## +α
v
## 4
α
## 15
## +α
## 14
## +α
## 10
## +α
## 8
## +α
## 7
## +α
v
## 5
α
## 15
## +α
## 14
## +α
## 13
## +α
## 11
## +α
## 10
## +α
## 8
## +α
## 5
## +α
## 3
## +α
## 2
## +α
v
## 6
α
## 15
## +α
## 12
## +α
## 8
## +α
## 6
## +α
## 3
## +α
## 2
v
## 7
α
## 14
## +α
## 4
## +α
v
## 8
α
## 14
## +α
## 13
## +α
## 11
## +α
## 10
## +α
## 7
## +α
## 4
## +α
## 3
v
## 9
α
## 12
## +α
## 7
## +α
## 6
## +α
## 4
## +α
## 3
v
## 10
α
## 14
## +α
## 13
## +α
## 11
## +α
## 9
## +α
## 6
## +α
## 5
## +α
## 4
## +α
v
## 11
α
## 15
## +α
## 13
## +α
## 12
## +α
## 11
## +α
## 8
v
## 12
α
## 15
## +α
## 14
## +α
## 13
## +α
## 12
## +α
## 11
## +α
## 10
## +α
## 8
## +α
## 7
## +α
## 5
## +α
## 4
## +α
## 3
v
## 13
α
## 15
## +α
## 14
## +α
## 13
## +α
## 12
## +α
## 11
## +α
## 9
## +α
## 8
## +α
## 5
## +α
## 4
## +α
## 2
v
## 14
α
## 15
## +α
## 14
## +α
## 13
## +α
## 12
## +α
## 11
## +α
## 10
## +α
## 9
## +α
## 8
## +α
## 5
## +α
## 4
## +α
## 3
v
## 15
α
## 15
## +α
## 12
## +α
## 11
## +α
## 8
## +α
## 4
## +α
## 3
## +α
## 2
## +α
Every message wordm
i
## =m
i,15
## ...m
i,0
consists of 16 bits. As such it could be regarded as binary vector of length 16:
(H.8)m
i
## =(m
i,0
## ...m
i,15
## )
## Wherem
i,0
is the least significant bit of message wordm
i
. Accordingly we consider the field element ̃m
i
## =
## ∑
## 15
j=0
m
i,j
v
j
to represent that message word.
Similarly, we assign a unique index to each validator between 0 and 1,022 and we represent validatoriwith the field
element:
## (H.9)
## ̃
i=
## 15
## ∑
j=0
i
j
v
j
wherei=i
## 15
## ...i
## 0
is the binary representation ofi.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202564
H.3.The Generator Polynomial.To erasure code a message of 342 words into 1023 code words, we represent each
message as a field element as described in previous section and we interpolate the polynomialp(y)of maximum 341
degree which satisfies the following equalities:
## (H.10)
p(
## ̃
## 0)=
## È
m
## 0
p(
## ̃
## 1)=
## È
m
## 1
## ⋮
p(
## É
## 341)=
## Ê
m
## 341
After findingp(y)with such properties, we evaluatepat the following points:
## (H.11)
## É
r
## 342
## ∶=p(
## É
## 342)
## É
r
## 343
## ∶=p(
## É
## 343)
## ⋮
## Ê
r
## 1022
## ∶=p(
## Ê
## 1022)
We then distribute the message words and the extra code words among the validators according to their corresponding
indices.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202565
AppendixI.Index of Notation
I.1.Sets.
I.1.1.Regular Notation.
F:The set of finite fields.
N:The set of non-negative integers. Subscript denotes one greater than the maximum. See section3.4.
## N
## +
## :
The set of positive integers (not including zero).
## N
## B
:The set of balance values. Equivalent toN
## 2
## 64
. See equation4.21.
## N
## G
:The set of unsigned gas values. Equivalent toN
## 2
## 64
. See equation4.23.
## N
## L
:The set of blob length values. Equivalent toN
## 2
## 32
. See section3.4.
## N
## R
:The set of register values. Equivalent toN
## 2
## 64
. See equation4.23.
## N
## S
:The set from which service indices are drawn. Equivalent toN
## 2
## 32
. See section9.1.
## N
## T
:The set of timeslot values. Equivalent toN
## 2
## 32
. See equation4.28.
Q:The set of rational numbers. Unused.
Z:The set of integers. Subscript denotes range. See section3.4.
## Z
## G
:The set of signed gas values. Equivalent toZ
## −2
## 63
## ...2
## 63
. See equation4.23.
I.1.2.Custom Notation.
jK→Vo:The set of dictionaries making a partial bijection of domainkto rangev. See section3.5.
A:The set of serviceAccounts. See equation9.3.
b:The set ofbitstrings (Boolean sequences). Subscript denotes length. See section3.7.
B:The set ofBlobs (octet sequences). Subscript denotes length. See section3.7.
## BLS
B:The set ofblspublic keys. A subset ofB
## 144
. See section3.8.2.
## ○
B:The set of Bandersnatch ring roots. A subset ofB
## 144
. See section3.8and appendixG.
C:The set of work-Contexts. See equation11.4.Not used as the set of complex numbers.
D:The set of work-Digests. See equation11.6.
E:The set of work executionErrors. See equation11.7.
G:The set representing the state of aGuestpvminstance. See equationB.4.
H:The set of 32-octet cryptographic values, equivalent toB
## 32
. Often aHash function’s result. See section3.8.
## ̄
H:The set of Ed25519 public keys. A subset ofB
## 32
. See section3.8.2.
## ∽
H:The set of Bandersnatch public keys. A subset ofB
## 32
. See section3.8and appendixG.
U:TheInformation concerning a single work-item once prepared as an operand for the accumulation function. See
equation
## 12.13.
J:The set of data segments, equivalent toB
## W
## G
. See equation14.1.
K:The set of validatorKey-sets. See equation6.7.
L:The set representing implications of accumulation. See equationB.7.
M:The set ofpvmMemory (ram) states. See equation4.24.
P:The set of work-Packages. See equation14.2.
R:The set of work-Reports. See equation11.2.Note used for the set of real numbers.
S:The set representating a portion of overallState, used during accumulation. See equation12.16.
T:The set of seal-keyTickets. See equation6.6.
## V
μ
:The set ofValidly readable indices forpvm ramμ. See appendixA.
## V
## ∗
μ
:The set ofValidly writable indices forpvm ramμ. See appendixA.
## ̄
## V
k
⟨m⟩:The set ofValid Ed25519 signatures of the keykand messagem. A subset ofB
## 64
. See section3.8.
## ∽
## V
m
k
⟨c⟩:The set ofValid Bandersnatch signatures of the public keyk, contextcand messagem. A subset ofB
## 96
## .
See section3.8.
## ○
## V
m
r
⟨c⟩:The set ofValid Bandersnatch Ringvrfproofs of the rootr, contextcand messagem. A subset ofB
## 784
## .
See section3.8.
W:The set ofWork items. See equation14.3.
X:The set of deferred transfers. See equation12.14.
Y:The set of availability specifications. See equation11.5.
I.2.Functions.
∆:The accumulation functions (see section12.2):
## ∆
## 1
:The single-step accumulation function. See equation12.24.
## ∆
## ∗
:The parallel accumulation function. See equation12.19.
## ∆
## +
:The full sequential accumulation function. See equation12.18.
Λ:The historical lookup function. See equation9.7.
Ξ:The work-report computation function. See equation14.12.
Υ:The general state transition function. See equations4.1,4.5.
Φ:The key-nullifier function. See equation6.14.
## Ψ
## :
The whole-program
pvm
machine state-transition function. See equationA.
## Ψ
## 1
:The single-step (pvm) machine state-transition function. See appendixA.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202566
## Ψ
## A
:The Accumulatepvminvocation function. See appendixB.
## Ψ
## H
:The host-function invocation (pvm) with host-function marshalling. See appendixA.
## Ψ
## I
:The Is-Authorizedpvminvocation function. See appendixB.
## Ψ
## M
:The marshalling whole-programpvmmachine state-transition function. See appendixA.
## Ψ
## R
:The Refinepvminvocation function. See appendixB.
Ω:Virtual machine host-call functions. See appendixB.
## Ω
## A
## :
Assign-core host-call.
## Ω
## B
:Empower-service host-call.
## Ω
## C
:Checkpoint host-call.
## Ω
## D
:Designate-validators host-call.
## Ω
## E
:Export segment host-call.
## Ω
## F
:Forget-preimage host-call.
## Ω
## G
:Gas-remaining host-call.
## Ω
## H
:Historical-lookup-preimage host-call.
## Ω
## I
:Information-on-service host-call.
## Ω
## J
:Eject-service host-call.
## Ω
## K
:Kickoff-pvmhost-call.
## Ω
## L
:Lookup-preimage host-call.
## Ω
## M
:Make-pvmhost-call.
## Ω
## N
:New-service host-call.
## Ω
## O
:Poke-pvmhost-call.
## Ω
## P
:Peek-pvmhost-call.
## Ω
## Q
:Query-preimage host-call.
## Ω
## R
:Read-storage host-call.
## Ω
## S
:Solicit-preimage host-call.
## Ω
## T
:Transfer host-call.
## Ω
## U
:Upgrade-service host-call.
## Ω
## W
:Write-storage host-call.
## Ω
## X
:Expunge-pvmhost-call.
## Ω
## Y
:Fetch data host-call.
## Ω
## Z
:Pages inner-pvmmemory host-call.
## Ω
## Q
:Yield accumulation trie result host-call.
## Ω
## P
:Provide preimage host-call.
I.3.Utilities, Externalities and Standard Functions.
A(...):The Merkle mountain range append function. See equationE.8.
## B
n
(...):The octets-to-bits function fornoctets. Superscripted
## −1
to denote the inverse. See equationA.12.
## C
n
(...):The erasure-coding functions fornchunks. See equationH.4.
E(...):The octet-sequence encode function. Superscripted
## −1
to denote the inverse. See appendixC.
F(...):The Fisher-Yates shuffle function. See equationF.1.
H(...):The Blake 2b 256-bit hash function. See section3.8.
## H
## K
(...):The Keccak 256-bit hash function. See section3.8.
## J
x
:The justification path to a specific2
x
size page of a constant-depth Merkle tree. See equationE.5.
K(...):The domain, or set of keys, of a dictionary. See section3.5.
## L
x
:The2
x
size page function for a constant-depth Merkle tree. See equationE.6.
M(...):The constant-depth binary Merklization function. See appendixE.
## M
## B
(...):The well-balanced binary Merklization function. See appendixE.
## M
σ
(...):The state Merklization function. See appendixD.
O(...):The Bandersnatch ring root function. See section3.8and appendixG.
## P
n
(...):The octet-array zero-padding function. See equation14.18.
Q(...):The numeric-sequence-from-hash function. See equationF.3.
R(...):The group of erasure-coding piece-recovery functions. See equationH.5.
## ̄
S(...):The Ed25519 signing function. See section3.8.
## BLS
S(...):Theblssigning function. See section3.8.
T:The current time expressed in seconds after the start of the
## J
amCommon Era. See section4.4.
U(...):The substitute-if-nothing function. See equation3.2.
V(...):The range, or set of values, of a dictionary or sequence. See section3.5.
## X
n
(...):The signed-extension function for a value inN
## 2
## 8n
. See equationA.16.
Y(...):The alias/output/entropy function of a Bandersnatchvrfsignature/proof. See section3.8and appendix
## G.
## Z
n
(...):The into-signed function for a value inN
## 2
## 8n
. Superscripted with
## −1
to denote the inverse. See equation
## A.10.
I.4.Values.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202567
I.4.1.Block-context Terms.These terms are all contextualized to a single block. They may be superscripted with some
other term to alter the context and reference some other block.
A:The ancestor set of the block. See equation5.3.
B:The block. See equation4.2.
E:The block extrinsic. See equation4.3.
## F
v
:TheBeefysigned commitment of validatorv. See equation18.1.
G:The set of Ed25519 guarantor keys who made a work-report. See equation11.26.
H:The block header. See equation5.1.
S:The sequence of work-reports which were accumulated this in this block. See equations12.28and12.29.
M:The mapping from cores to guarantor keys. See section11.3.
## M
## ∗
:The mapping from cores to guarantor keys for the previous rotation. See section11.3.
R:The sequence of work-reports which have now become available and ready for accumulation. See equation11.16.
T:The ticketed condition, true if the block was sealed with a ticket signature rather than a fallback. See equations
## 6.15and6.16.
U:The audit condition, equal to⊺once the block is audited. See section17.
Without any superscript, the block is assumed to the block being imported or, if no block is being imported, the head
of the best chain (see section19). Explicit block-contextualizing superscripts include:
## B
## ♮
:The latest finalized block. See equation19.
## B
## ♭
:The block at the head of the best chain. See equation19.
I.4.2.State components.Here, the prime annotation indicates posterior state. Individual components may be identified
with a letter subscript.
α:The coreαuthorizations pool. See equation8.1.
β:Log of recent activity. See equation7.1.
β
## H
:Information on the most recent blocks. See equation7.2.
β
## B
:The Merkle mountain belt for accumulating Accumulation outputs. See equations7.3and7.7.
γ:State concerning Safrole. See equation6.3.
γ
## A
:The sealing lottery ticket accumulator. See equation6.5.
γ
## P
:The keys for the validators of the next epoch, equivalent to those keys which constituteγ
## Z
. See equation
## 6.7.
γ
## S
:The sealing-key sequence of the current epoch. See equation6.5.
γ
## Z
:The Bandersnatch root for the current epoch’s ticket submissions. See equation6.4.
δ:The (prior) state of the service accounts. See equation9.1.
δ
## †
:The post-accumulation, pre-preimage integration intermediate state. See equation12.27.
η:The entropy accumulator and epochal randomness. See equation6.21.
ι:The validator keys and metadata to be drawn from next. See equation6.7.
κ:The validator keys and metadata currently active. See equation6.7.
λ:The validator keys and metadata which were active in the prior epoch. See equation6.7.
ρ:The pending reports, per core, which are being made available prior to accumulation. See equation11.1.
ρ
## †
:The post-judgment, pre-guarantees-extrinsic intermediate state. See equation10.15.
ρ
## ‡
:The post-guarantees-extrinsic, pre-assurances-extrinsic, intermediate state. See equation11.17.
σ:The overall state of the system. See equations4.1,4.4.
τ:The most recent block’s timeslot. See equation6.1.
φ:The authorization queue. See equation8.1.
ψ:Past judgments on work-reports and validators. See equation10.1.
ψ
## B
:Work-reports judged to be incorrect. See equation10.17.
ψ
## G
:Work-reports judged to be correct. See equation10.16.
ψ
## W
:Work-reports whose validity is judged to be unknowable. See equation10.18.
ψ
## O
:Validators who made a judgment found to be incorrect. See equation10.19.
χ:The privileged service indices. See equation9.9.
χ
## M
:The index of the blessed service. See equation12.27.
χ
## A
:The indices of the services able to assign each core’s authorizer queue. See equation12.27.
χ
## V
:The index of the designate service. See equation12.27.
χ
## R
:The index of the registrar service. See equation12.27.
χ
## Z
:The always-accumulate service indices and their basic gas allowance. See equation12.27.
π:The activity statistics for the validators. See equation13.1.
ω:The accumulation queue. See equation12.3.
ξ:The accumulation history. See equation12.1.
θ:The most recent Accumulation outputs. See equations7.4and12.25.
I.4.3.Virtual Machine components.
ε
## :
The exit-reason resulting from all machine state transitions.
ν:The immediate values of an instruction.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202568
μ:The memory sequence; a member of the setM.
ρ:The gas counter.
φ:The registers.
ζ:The instruction sequence.
π:The sequence of basic blocks of the program.
ı:The instruction counter.
I.4.4.Constants.
A=8:The period, in seconds, between audit tranches. See section17.3.
## B
## I
=10:The additional minimum balance required per item of elective service state. See equation9.8.
## B
## L
=1:The additional minimum balance required per octet of elective service state. See equation9.8.
## B
## S
=100:The basic minimum balance which all services require. See equation9.8.
C=341:The total number of cores.
D=19,200:The period in timeslots after which an unreferenced preimage may be expunged. Seeejectdefinition
in sectionB.7.
E=600:The length of an epoch in timeslots. See section4.8.
F=2:The audit bias factor, the expected number of additional validators who will audit a work-report in the
following tranche for each no-show in the previous. See equation17.14.
## G
## A
=10,000,000:The gas allocated to invoke a work-report’s Accumulation logic.
## G
## I
=50,000,000:The gas allocated to invoke a work-package’s Is-Authorized logic.
## G
## R
=5,000,000,000:The gas allocated to invoke a work-package’s Refine logic.
## G
## T
=3,500,000,000:The total gas allocated across for all Accumulation. Should be no smaller thanG
## A
## ⋅C+
## ∑
g∈V(χ
## Z
## )
## (g).
H=8:The size of recent history, in blocks. See equation7.8.
I=16:The maximum amount of work items in a package. See equations11.2and14.2.
J=8:The maximum sum of dependency items in a work-report. See equation11.3.
K=16:The maximum number of tickets which may be submitted in a single extrinsic. See equation6.30.
L=14,400:The maximum age in timeslots of the lookup anchor. See equation11.34.
N=2:The number of ticket entries per validator. See equation6.29.
O=8:The maximum number of items in the authorizations pool. See equation8.1.
P=6:The slot period, in seconds. See equation4.8.
Q=80:The number of items in the authorizations queue. See equation8.1.
R=10:The rotation period of validator-core assignments, in timeslots. See sections11.3and11.4.
## S=2
## 16
:The minimum public service index. Services of indices below these may only be created by the Registrar.
See equation
## B.14.
T=128:The maximum number of extrinsics in a work-package. See equation14.4.
U=5:The period in timeslots after which reported but unavailable work may be replaced. See equation11.17.
V=1023:The total number of validators.
## W
## A
=64,000:The maximum size of is-authorized code in octets. See equationB.1.
## W
## B
=13,791,360:The maximum size of the concatenated variable-size blobs, extrinsics and imported segments of
a work-package, in octets. See equation14.5.
## W
## C
=4,000,000:The maximum size of service code in octets. See equationsB.5,B.9&??.
## W
## E
=684:The basic size of erasure-coded pieces in octets. See equationH.4.
## W
## G
## =W
## P
## W
## E
=4104:The size of a segment in octets. See section14.2.1.
## W
## F
## =W
## G
## +32⌈log
## 2
## (W
## M
)⌉=4488:The additional footprint in the Audits DA of a single imported segment. See
equation
## 14.6.
## W
## M
=3,072:The maximum number of imports in a work-package. See equation14.4.
## W
## P
=6:The number of erasure-coded pieces in a segment.
## W
## R
## =48⋅2
## 10
:The maximum total size of all unbounded blobs in a work-report, in octets. See equation11.8.
## W
## T
=128:The size of a transfer memo in octets. See equation12.14.
## W
## X
=3,072:The maximum number of exports in a work-package. See equation14.4.
X:Context strings, see below.
Y=500:The number of slots into an epoch at which ticket-submission ends. See sections6.5,6.6and6.7.
## Z
## A
=2:Thepvmdynamic address alignment factor. See equationA.18.
## Z
## I
## =2
## 24
:The standardpvmprogram initialization input data size. See equationA.7.
## Z
## P
## =2
## 12
:Thepvmmemory page size. See equation4.24.
## Z
## Z
## =2
## 16
:The standardpvmprogram initialization zone size. See sectionA.7.
I.4.5.Signing Contexts.
## X
## A
=$jam_available:Ed25519Availability assurances. See equation11.13.
## X
## B
=$jam_beefy:blsAccumulate-result-root-mmrcommitment. See equation18.1.
## X
## E
=$jam_entropy:On-chain entropy generation. See equation6.17.
## X
## F
## =
## $jam_fallback_seal
:Bandersnatch
Fallback block seal. See equation
## 6.16.
## X
## G
=$jam_guarantee:Ed25519Guarantee statements. See equation11.26.

JAM: JOIN-ACCUMULATE MACHINEDRAFT 0.7.2 - September 15, 202569
## X
## I
=$jam_announce:Ed25519Audit announcement statements. See equation17.8.
## X
## T
=$jam_ticket_seal:Bandersnatch RingvrfTicket generation and regular block seal. See equation6.15.
## X
## U
=$jam_audit:BandersnatchAudit selection entropy. See equations17.3and17.14.
## X
## ⊺
=$jam_valid:Ed25519Judgments for valid work-reports. See equation17.17.
## X
## 
=$jam_invalid:Ed25519Judgments for invalid work-reports. See equation17.17.

## REFERENCES70
## References
Bertoni, Guido et al. (2013). “Keccak”. In:Annual international conference on the theory and applications of cryptographic
techniques. Springer, pp. 313–314.
Bögli, Roman (2024). “AssessingriscZero using ZKit: An Extensible Testing and Benchmarking Suite for ZKP Frame-
works”. PhD thesis. OST Ostschweizer Fachhochschule.
Boneh, Dan, Ben Lynn, and Hovav Shacham (2004). “Short Signatures from the Weil Pairing”. In:J. Cryptology17,
pp. 297–319.doi:10.1007/s00145-004-0314-9.
Burdges, Jeff, Alfonso Cevallos, et al. (2024).Eﬀicient Execution Auditing for Blockchains under Byzantine Assumptions.
Cryptology ePrint Archive, Paper 2024/961.https://eprint.iacr.org/2024/961.url:https://eprint.iacr.org/
## 2024/961.
Burdges, Jeff, Oana Ciobotaru, et al. (2022).Eﬀicient Aggregatable BLS Signatures with Chaum-Pedersen Proofs. Cryp-
tology ePrint Archive, Paper 2022/1611.https://eprint.iacr.org/2022/1611.url:https://eprint.iacr.org/
## 2022/1611.
Burdges, Jeffrey et al. (2023).Ring Verifiable Random Functions and Zero-Knowledge Continuations. Cryptology ePrint
## Archive, Paper 2023/002.url:https://eprint.iacr.org/2023/002.
Buterin, Vitalik (2013).Ethereum: A Next-Generation Smart Contract and Decentralized Application Platform.url:
https://github.com/ethereum/wiki/wiki/White-Paper.
Buterin, Vitalik and Virgil Griﬀith (2019).Casper the Friendly Finality Gadget. arXiv:1710.09437 [cs.CR].
Cosmos Project (2023).Interchain Security Begins a New Era for Cosmos. Fetched 18th March, 2024.url:https:
//blog.cosmos.network/interchain-security-begins-a-new-era-for-cosmos-a2dc3c0be63.
Dune and hildobby (2024).Ethereum Staking. Fetched 18th March, 2024.url:https://dune.com/hildobby/eth2-
staking
## .
Ethereum Foundation (2024a). “A digital future on a global scale”. In: Fetched 4th April, 2024.url:https://ethereum.
org/en/roadmap/vision/.
— (2024b).Danksharding. Fetched 18th March, 2024.url:https://ethereum.org/en/roadmap/danksharding/.
Fisher, Ronald Aylmer and Frank Yates (1938).Statistical tables for biological, agricultural and medical research. Oliver
and Boyd.
Gabizon, Ariel, Zachary J. Williamson, and Oana Ciobotaru (2019).PLONK: Permutations over Lagrange-bases for
Oecumenical Noninteractive arguments of Knowledge. Cryptology ePrint Archive, Paper 2019/953.url:https://
eprint.iacr.org/2019/953.
Goldberg, Sharon et al. (Aug. 2023).Verifiable Random Functions (VRFs). RFC 9381.doi:10.17487/RFC9381.url:
https://www.rfc-editor.org/info/rfc9381.
Hertig, Alyssa (2016).So, Ethereum’s Blockchain is Still Under Attack...Fetched 18th March, 2024.url:https:
//www.coindesk.com/markets/2016/10/06/so-ethereums-blockchain-is-still-under-attack/.
Hopwood, Daira et al. (2020).BLS12-381.url:https://z.cash/technology/jubjub/.
Hosseini, Seyed and Davide Galassi (2024). “Bandersnatch VRF-AD Specification”. In: Fetched 10th March, 2025.url:
https://github.com/davxy/bandersnatch-vrfs-spec/blob/main/specification.pdf.
Jha, Prashant (2024).Solana outage raises questions about client diversity and beta status. Fetched 18th March, 2024.
url:https://cointelegraph.com/news/solana-outage-client-diversity-beta.
Josefsson, Simon and Ilari Liusvaara (Jan. 2017).Edwards-Curve Digital Signature Algorithm (EdDSA). RFC 8032.doi:
10.17487/RFC8032.url:https://www.rfc-editor.org/info/rfc8032.
Kokoris-Kogias, Eleftherios et al. (2017).OmniLedger: A Secure, Scale-Out, Decentralized Ledger via Sharding. Cryptology
ePrint Archive, Paper 2017/406.
https://eprint.iacr.org/2017/406.
url
## :https://eprint.iacr.org/2017/406.
Kwon, Jae and Ethan Buchman (2019). “Cosmos whitepaper”. In:A Netw. Distrib. Ledgers27, pp. 1–32.
Lin, Sian-Jheng, Wei-Ho Chung, and Yunghsiang S. Han (2014). “Novel Polynomial Basis and Its Application to Reed-
Solomon Erasure Codes”. In:2014 IEEE 55th Annual Symposium on Foundations of Computer Science, pp. 316–325.
doi:
## 10.1109/FOCS.2014.41.
Masson, Simon, Antonio Sanso, and Zhenfei Zhang (2021).Bandersnatch: a fast elliptic curve built over the BLS12-381
scalar field. Cryptology ePrint Archive, Paper 2021/1152.url:https://eprint.iacr.org/2021/1152.
Ng, Felix (2024).Is measuring blockchain transactions per second stupid in 2024?Fetched 18th March, 2024.url:
https://cointelegraph.com/magazine/blockchain-transactions-per-second-tps-stupid-big-questions/.
PolkavmProject (2024). “PolkaVM/RISC0 Benchmark Results”. In: Fetched 3rd April, 2024.url:https://github.
com/koute/risc0-benchmark/blob/master/README.md.
Saarinen, Markku-Juhani O. and Jean-Philippe Aumasson (Nov. 2015).The BLAKE2 Cryptographic Hash and Message
Authentication Code (MAC). RFC 7693.doi:10.17487/RFC7693.url:https://www.rfc-editor.org/info/rfc7693.
Sadana, Apoorv (2024).Bringing Polkadot tech to Ethereum. Fetched 18th March, 2024.url:https://ethresear.ch/
t/bringing-polkadot-tech-to-ethereum/17104.
Sharma, Shivam (2023).Ethereum’s Rollups are Centralized.url:https://public.bnbstatic.com/static/files/
research/ethereums-rollups-are-centralized-a-look-into-decentralized-sequencers.pdf.
Solana Foundation (2023).Solana data goes live on Google Cloud BigQuery. Fetched 18th March, 2024.url:https:
## //solana.com/news/solana-data-live-on-google-cloud-bigquery
## .

## REFERENCES71
Solana Labs (2024).Solana Validator Requirements. Fetched 18th March, 2024.url:https://docs.solanalabs.com/
operations/requirements.
Stewart, Alistair and Eleftherios Kokoris-Kogia (2020). “Grandpa: a byzantine finality gadget”. In:arXiv preprint
arXiv:2007.01560.
Tanana, Dmitry (2019). “Avalanche blockchain protocol for distributed computing security”. In:2019 IEEE International
Black Sea Conference on Communications and Networking (BlackSeaCom). IEEE, pp. 1–3.
Thaler, Justin (2023). “A technical FAQ on Lasso, Jolt, and recent advancements in SNARK design”. In: Fetched 3rd
April, 2024.url:https://a16zcrypto.com/posts/article/a-technical-faq-on-lasso-jolt-and-recent-
advancements-in-snark-design/.
Wikipedia (2024).Fisher-Yates shuffle: The modern algorithm.url:https://en.wikipedia.org/wiki/Fisher%5C%E2%
5C%80%5C%93Yates_shuffle%5C#The_modern_algorithm.
Wood, Gavin (2014). “Ethereum: A secure decentralised generalised transaction ledger”. In:Ethereum project yellow
paper151, pp. 1–32.
Yakovenko, Anatoly (2018). “Solana: A new architecture for a high performance blockchain v0. 8.13”. In.





