# ZP1 Cryptographic Concepts

This document explains the core mathematical and cryptographic primitives that power the ZP1 zkVM.

## 1. Mersenne-31 (M31)

**Definition**: A finite field $\mathbb{F}_p$ where the prime $p = 2^{31} - 1 = 2147483647$.

**Why ZP1 uses it**:
*   **Performance**: It is the largest prime that fits into a signed 32-bit integer. Addition is extremely fast (just a regular add + conditional subtract). Multiplication fits in a 64-bit register without overflow.
*   **Hardware Compatibility**: Native to CPUs (x86/ARM) and highly efficient on GPUs, which often favor 32-bit operations.

**The Catch**: $p-1 = 2 \cdot 3 \cdot 715827882$. It is not divisible by a large power of 2. This means standard FFT algorithms (which require $2^k | p-1$) cannot be used directly.

## 2. Circle STARKs

**Definition**: A STARK proving system that uses the group of points on the unit circle $x^2 + y^2 = 1$ over $\mathbb{F}_p$ instead of the multiplicative group of the field.

**How it works**:
*   While the field's multiplicative group size ($p-1$) is "bad" for FFTs, the number of points on the circle group is $p + 1 = 2^{31} = 2,147,483,648$.
*   Since $2^{31}$ is a huge power of 2, we *can* perform efficient FFTs over this circle group.
*   **ZP1 Implementation**: We map our execution trace to points on this circle, allowing us to use O(N log N) proving algorithms even though we are using the "FFT-unfriendly" M31 field.

## 3. QM31 (Degree 4 Extension Field)

**Definition**: QM31 is a secure extension field formed by extending M31 four times (Degree 4).
*   **Notation**: $\mathbb{F}_{p^4}$
*   **Size**: $\approx 2^{124}$

**Why we need it**:
*   **Security**: The base field M31 ($2^{31}$) is too small for cryptographic security. An attacker could find collisions with $\approx 2^{15.5}$ work (birthday paradox).
*   **Usage**: We run the main computation in the fast M31 field. When we need to generate random "challenges" (to verify the computation was correct), we sample them from QM31. This provides $\approx 100+$ bits of security against attacks.

## 4. FFT (Fast Fourier Transform)

**Definition**: An algorithm that converts polynomials between two forms:
1.  **Coefficient Form**: $f(x) = a_0 + a_1x + a_2x^2 + \dots$
2.  **Point-Value Form**: Evaluations of $f(x)$ at specific points.

**In ZP1**:
*   **Circle FFT**: A variant of the FFT adapted for the circle group.
*   **Role**: It is the engine of the prover. It allows us to:
    *   Extend the execution trace (Low-Degree Extension).
    *   Compute the quotient polynomial (combining all constraints).
    *   Without FFTs, proving would be O(N²) and impractically slow. With Circle FFT, it is **O(N log N)**.

## 5. DEEP FRI

**Definition**: A combination of two protocols used to commit to polynomials and prove they are low-degree.

*   **DEEP (Domain Extension for Eliminating Pretenders)**: A technique to check the consistency of polynomials at a random point $z$ sampled from outside the trace domain. It prevents "pretender" polynomials from tricking the verifier.
*   **FRI (Fast Reed-Solomon Interactive Oracle Proof)**: The protocol that proves a committed polynomial is actually a polynomial of low degree (and not just random noise).
    *   It strictly reduces the problem size by folding the polynomial in half repeatedly ($N \to N/2 \to N/4 \dots$).
    *   Eventually, it becomes small enough to send directly to the verifier.

**Summary in ZP1**:
The **Prover** uses **Circle FFTs** to manipulate polynomials over **M31**, extends them to provided security via **QM31** challenges, and uses **DEEP FRI** to succinctly prove to the verifier that the computation was correct.
