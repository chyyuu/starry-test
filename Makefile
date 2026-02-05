# Build Options
export ARCH := riscv64
export LOG := warn
export DWARF := y
export MEMTRACK := n
export INIT_CMDLINE ?=

# QEMU Options (Minimal for ch18_file0)
export BLK := y
export NET := n
export VSOCK := n
export MEM := 128M
export ICOUNT := n

# Generated Options (Minimal for ch18_file0)
export A := $(PWD)
export NO_AXSTD := y
export AX_LIB := axfeat
export APP_FEATURES := qemu

ifeq ($(MEMTRACK), y)
	APP_FEATURES += starry-api/memtrack
endif

default: build

ROOTFS_URL = https://github.com/Starry-OS/rootfs/releases/download/20250917
ROOTFS_IMG = rootfs-$(ARCH).img

rootfs:
	@if [ ! -f $(ROOTFS_IMG) ]; then \
		echo "Image not found, downloading..."; \
		curl -f -L $(ROOTFS_URL)/$(ROOTFS_IMG).xz -O; \
		xz -d $(ROOTFS_IMG).xz; \
	fi
	@cp $(ROOTFS_IMG) arceos/disk.img

img:
	@echo -e "\033[33mWARN: The 'img' target is deprecated. Please use 'rootfs' instead.\033[0m"
	@$(MAKE) --no-print-directory rootfs

minimal-rootfs:
	@echo "Building minimal rootfs for ch18_file0..."
	@mkdir -p minimal_fs/dev
	@if [ -f linux-app/ch18_file0 ]; then \
		cp linux-app/ch18_file0 minimal_fs/; \
		chmod +x minimal_fs/ch18_file0; \
	else \
		echo "Error: linux-app/ch18_file0 not found"; \
		exit 1; \
	fi
	@echo "Creating 8MB ext4 filesystem..."
	@mkfs.ext4 -d minimal_fs -F arceos/disk.img 8M
	@rm -rf minimal_fs
	@echo "Minimal rootfs created at arceos/disk.img"

defconfig justrun clean:
	@make -C arceos $@

build run debug disasm: defconfig
	@make -C arceos $@

# Aliases
rv:
	$(MAKE) ARCH=riscv64 run

la:
	$(MAKE) ARCH=loongarch64 run

vf2:
	$(MAKE) ARCH=riscv64 APP_FEATURES=vf2 MYPLAT=axplat-riscv64-visionfive2 BUS=mmio build

.PHONY: build run justrun debug disasm clean
