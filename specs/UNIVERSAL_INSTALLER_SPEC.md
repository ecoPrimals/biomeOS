# `biomeOS` - Universal Installer Specification v1

**Status:** Draft | **Author:** The Architect & The Artisan AI | **Date:** July 2025

**Related Documents:** [BOOTSTRAP_UI_SPEC.md](./BOOTSTRAP_UI_SPEC.md)

---

## 1. Preamble: The Universal Onboard

To achieve true sovereignty and accessibility, `biomeOS` requires a universal installation method that transcends platform-specific formats like `.exe` or `.deb`. This "universal onboard" must be simple enough for any user and robust enough for any target system, from bare metal to an existing desktop.

This specification defines the architecture of the `biomeOS` universal installer, which takes the form of a single, bootable **ISO image**.

## 2. The Solution: A Dual-Purpose ISO Image

The distributable artifact will be a single file: `biomeOS-installer.iso`. This file is designed to be written to a USB drive or mounted on a host OS, providing two distinct installation pathways from one source.

### 2.1. ISO Contents and Build Process

The ISO will be constructed by combining a minimal boot environment with the `biomeOS` components.

```mermaid
graph TD
    subgraph "Source Code Repositories"
        A[biomeos-ui]
        B[toadstool]
        C[songbird]
        D[beardog]
    end

    subgraph "Build Process"
        E[Compile `biomeos-ui` for Linux/x86_64] --> F{ISO Root}
        G[Compile `biomeos-ui` for Windows/x86_64] --> H[/installers/windows/]
        I[Compile `biomeos-ui` for Linux/x86_64 (again)] --> J[/installers/linux/]
        K[Package Primals (B, C, D)] --> F
    end

    subgraph "Minimal Boot Environment"
        L[Linux Kernel] --> F
        M[Bootloader (GRUB2)] --> F
        N[Initramfs] --> F
    end

    F & H & J --> O(biomeOS-installer.iso)

    style F fill:#f9f,stroke:#333,stroke-width:2px
    style O fill:#ccf,stroke:#333,stroke-width:4px
```

The ISO will contain:
-   **A Bootloader:** GRUB2, configured for both legacy BIOS and modern UEFI booting.
-   **A Linux Kernel:** A minimal, modern Linux kernel to handle the low-level hardware initialization. This is a pragmatic choice to avoid the immense complexity of writing a custom kernel and drivers from scratch.
-   **An Initramfs:** A small initial root filesystem containing the bare necessities to run our installer.
-   **The `biomeos-ui` Installer:** The pure-Rust `egui` application, compiled for the minimal Linux environment. This is the main application launched on boot.
-   **Primal Packages:** The compiled binaries for `toadstool`, `songbird`, `beardog`, etc., ready to be installed.
-   **Host Installers:** A dedicated `/installers` directory containing pre-compiled versions of `biomeos-ui` for Windows and Linux, for use on existing systems.

## 3. Installation Workflows

### 3.1. Use Case 1: Bare-Metal Installation
1.  **Action:** User burns the `.iso` to a USB drive and boots a machine from it.
2.  **Process:** The bootloader loads the kernel, which executes the `biomeos-ui` application from the initramfs.
3.  **UI Flow:** The `egui` interface launches in "Bare Metal Mode." It guides the user through:
    -   Selecting a target hard drive.
    -   Confirming disk partitioning and formatting.
    -   Copying the `biomeOS` Primals to the new partitions.
    -   Installing the bootloader onto the target drive.
4.  **Result:** A fully sovereign, bootable `biomeOS` machine.

### 3.2. Use Case 2: Installation on a Host OS (Windows/Linux)
1.  **Action:** User mounts the `.iso` on their existing desktop.
2.  **Process:** They navigate to the `/installers/` directory and run the version of `biomeos-ui` for their platform.
3.  **UI Flow:** The `egui` interface launches in "Host OS Mode." It guides the user through:
    -   Confirming the installation directory.
    -   Copying the Primal binaries to the local filesystem.
    -   Registering the Primals as background services (e.g., `systemd` services on Linux, Windows Services on Windows).
4.  **Result:** `biomeOS` runs as a powerful, sovereign service layer on the user's existing operating system. 