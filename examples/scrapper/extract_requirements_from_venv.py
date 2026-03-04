#!/usr/bin/env python3
"""
Extract requirements.txt from existing virtual environment
This script scans a venv folder and generates a requirements.txt file
based on the actually installed packages.
"""

import os
import sys
import json
from pathlib import Path
from typing import List, Dict, Set, Tuple

def find_site_packages(venv_path: Path) -> Path:
    """Find the site-packages directory in the venv."""
    # Try different possible locations
    possible_paths = [
        venv_path / "lib" / f"python{sys.version_info.major}.{sys.version_info.minor}" / "site-packages",
        venv_path / "Lib" / "site-packages",  # Windows
        venv_path / "lib64" / f"python{sys.version_info.major}.{sys.version_info.minor}" / "site-packages",
    ]

    for path in possible_paths:
        if path.exists():
            return path

    # Search recursively for site-packages
    for root, dirs, files in os.walk(venv_path):
        if 'site-packages' in dirs:
            return Path(root) / 'site-packages'

    raise FileNotFoundError(f"Could not find site-packages in {venv_path}")

def get_installed_packages(site_packages: Path) -> Dict[str, str]:
    """Extract package names and versions from site-packages."""
    packages = {}

    # Look for .dist-info directories
    for item in site_packages.iterdir():
        if item.is_dir() and item.name.endswith('.dist-info'):
            # Parse package name and version from directory name
            # Format: package_name-version.dist-info
            dist_info_name = item.name[:-len('.dist-info')]

            # Try to read METADATA file
            metadata_file = item / 'METADATA'
            if metadata_file.exists():
                with open(metadata_file, 'r', encoding='utf-8', errors='ignore') as f:
                    name = None
                    version = None
                    for line in f:
                        if line.startswith('Name:'):
                            name = line.split(':', 1)[1].strip()
                        elif line.startswith('Version:'):
                            version = line.split(':', 1)[1].strip()
                        if name and version:
                            break

                    if name and version:
                        packages[name] = version
            else:
                # Fallback: parse from directory name
                parts = dist_info_name.split('-')
                if len(parts) >= 2:
                    name = parts[0]
                    version = parts[1]
                    packages[name] = version

    return packages

def get_package_dependencies(site_packages: Path, package_name: str) -> Set[str]:
    """Get direct dependencies of a package."""
    dependencies = set()

    # Find the .dist-info directory
    for item in site_packages.iterdir():
        if item.is_dir() and item.name.startswith(package_name.replace('-', '_') + '-') and item.name.endswith('.dist-info'):
            metadata_file = item / 'METADATA'
            if metadata_file.exists():
                with open(metadata_file, 'r', encoding='utf-8', errors='ignore') as f:
                    for line in f:
                        if line.startswith('Requires-Dist:'):
                            # Parse dependency name
                            dep = line.split(':', 1)[1].strip()
                            dep_name = dep.split()[0].split('(')[0].split('[')[0].split(';')[0].strip()
                            dependencies.add(dep_name.lower())
            break

    return dependencies

def filter_top_level_packages(packages: Dict[str, str], site_packages: Path) -> Dict[str, str]:
    """Filter to only include top-level packages (not their dependencies)."""
    all_deps = set()

    # Collect all dependencies
    for package_name in packages.keys():
        deps = get_package_dependencies(site_packages, package_name)
        all_deps.update(deps)

    # Filter out packages that are dependencies of other packages
    top_level = {}
    for name, version in packages.items():
        if name.lower() not in all_deps:
            top_level[name] = version

    return top_level

def generate_requirements(venv_path: str, output_file: str = 'requirements_from_venv.txt', include_all: bool = False):
    """Generate requirements.txt from venv folder."""
    venv_path = Path(venv_path).resolve()

    if not venv_path.exists():
        print(f"❌ Error: Virtual environment not found at {venv_path}")
        return

    print(f"🔍 Scanning virtual environment: {venv_path}")

    try:
        site_packages = find_site_packages(venv_path)
        print(f"✓ Found site-packages: {site_packages}")
    except FileNotFoundError as e:
        print(f"❌ Error: {e}")
        return

    print("\n📦 Extracting installed packages...")
    packages = get_installed_packages(site_packages)
    print(f"✓ Found {len(packages)} installed packages")

    if not include_all:
        print("\n🔍 Filtering to top-level packages only...")
        packages = filter_top_level_packages(packages, site_packages)
        print(f"✓ Filtered to {len(packages)} top-level packages")

    # Sort packages alphabetically
    sorted_packages = sorted(packages.items(), key=lambda x: x[0].lower())

    # Generate requirements.txt content
    requirements_lines = []
    requirements_lines.append("# Generated from virtual environment")
    requirements_lines.append(f"# Source: {venv_path}")
    requirements_lines.append(f"# Python {sys.version_info.major}.{sys.version_info.minor}")
    requirements_lines.append("")

    for name, version in sorted_packages:
        requirements_lines.append(f"{name}=={version}")

    # Write to file
    output_path = Path(output_file)
    with open(output_path, 'w', encoding='utf-8') as f:
        f.write('\n'.join(requirements_lines))

    print(f"\n✅ Requirements file generated: {output_path.absolute()}")
    print(f"\n📋 Preview (first 20 packages):")
    print("=" * 50)
    for line in requirements_lines[:24]:  # Show header + 20 packages
        print(line)
    if len(sorted_packages) > 20:
        print(f"... and {len(sorted_packages) - 20} more packages")
    print("=" * 50)

if __name__ == '__main__':
    import argparse

    parser = argparse.ArgumentParser(
        description='Extract requirements.txt from a Python virtual environment'
    )
    parser.add_argument(
        'venv_path',
        nargs='?',
        default='venv',
        help='Path to virtual environment folder (default: venv)'
    )
    parser.add_argument(
        '-o', '--output',
        default='requirements_from_venv.txt',
        help='Output file name (default: requirements_from_venv.txt)'
    )
    parser.add_argument(
        '-a', '--all',
        action='store_true',
        help='Include all packages (not just top-level)'
    )

    args = parser.parse_args()

    generate_requirements(args.venv_path, args.output, args.all)
