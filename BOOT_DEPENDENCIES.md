# BiomeOS Bootable Media - Dependencies

## Required Tools

To create bootable ISO/USB images, you need:

### grub-mkrescue (Primary Method - Recommended)

`grub-mkrescue` requires `xorriso` as its backend.

**Ubuntu/Debian**:
```bash
sudo apt install grub-common xorriso
```

**Arch Linux**:
```bash
sudo pacman -S grub xorriso
```

**Fedora/RHEL**:
```bash
sudo dnf install grub2-tools xorriso
```

### xorriso (Fallback Method)

If `grub-mkrescue` is not available, `xorriso` alone can create ISOs:

```bash
sudo apt install xorriso  # Ubuntu/Debian
sudo pacman -S xorriso     # Arch
sudo dnf install xorriso   # Fedora/RHEL
```

## Optional: Kernel Access

System kernels in `/boot/` require root access to read.

### Option 1: Copy kernel to accessible location

```bash
# Run the helper script (requires sudo once)
./scripts/prepare-kernel.sh

# Then build without sudo
export BIOMEOS_KERNEL=/tmp/vmlinuz-biomeos
cargo run --release -p biomeos-boot --bin biomeos-mkboot -- iso
```

### Option 2: Build with sudo

```bash
sudo -E cargo run --release -p biomeos-boot --bin biomeos-mkboot -- iso
```

**Note**: This preserves your environment variables (`-E` flag)

### Option 3: Use custom kernel

```bash
export BIOMEOS_KERNEL=/path/to/your/vmlinuz
cargo run --release -p biomeos-boot --bin biomeos-mkboot -- iso
```

## Current System Status

```bash
# Check if grub-mkrescue is available
which grub-mkrescue

# Check if xorriso is available  
which xorriso

# Check kernel access
ls -l /boot/vmlinuz
```

## Fallback: tar.gz Archive

If neither `grub-mkrescue` nor `xorriso` are available, BiomeOS will create a `tar.gz` archive containing all boot files. This is not bootable but preserves the data.

To create a bootable image from the tar.gz:

```bash
# Extract
mkdir boot-content
tar -xzf dist/biomeos-*.tar.gz -C boot-content

# Create ISO with grub-mkrescue
grub-mkrescue -o biomeos.iso boot-content/
```

