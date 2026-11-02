# Security Policy

Rusty Chain Kit takes the security of smart contract developer tooling, formal verification routines, and local blockchain execution environments seriously.

---

## Supported Versions

Security updates and patches are actively maintained for the following versions:

| Version | Supported |
| :--- | :--- |
| `0.1.x` (`main`) | Yes |
| `< 0.1.0` | No |

---

## Reporting a Vulnerability

If you discover a potential security vulnerability in Rusty Chain Kit, please report it responsibly rather than opening a public issue.

### Preferred Method
Please report vulnerabilities privately using GitHub Private Vulnerability Reporting:
1. Navigate to the repository's **Security** tab.
2. Under **Reporting**, click **Advisories** and select **Report a vulnerability**.
3. Include:
   - Type of vulnerability (e.g., parser panic, memory corruption, privilege escalation, state verification bypass).
   - Minimal reproduction steps, contract files, or test cases.
   - Any proposed remediation or patch.

### Direct Contact
Alternatively, you may contact the maintainer directly via email:
- **Security Contact**: Jeong Hoon Kim (`omakasekim@naver.com`)

---

## Response Timeline

- **Initial Response**: Within 48 hours of initial report receipt.
- **Triage & Assessment**: Within 5 business days, confirming exploitability and assigning severity.
- **Remediation & Disclosure**: Security patches are tested and published in a coordinated release. Public disclosure occurs after release availability to allow users to update their installations.
