# Threat Model

This document outlines the threat model for the Construct Messenger PWA.

*This is a placeholder document.*

## Adversaries

-   Malicious Server Administrator
-   Network Eavesdropper (Man-in-the-Middle)
-   Malicious Browser Extension
-   XSS Attacker
-   Physical Attacker with access to a user's device

## Threats

| Threat | Mitigation |
| --- | --- |
| Eavesdropping on messages | End-to-end encryption (Signal Protocol) |
| Server reading message content | E2EE ensures server only sees ciphertext |
| XSS stealing private keys | CSP, Web Worker isolation for crypto |
| Compromised dependency (npm) | Strict CSP, dependency auditing, lockfiles |
| IP Address leakage | TURN servers, disabling WebRTC features |
