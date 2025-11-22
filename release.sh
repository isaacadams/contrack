#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get version from Cargo.toml
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Cargo.toml not found${NC}"
    exit 1
fi

VERSION=$(grep -E '^\s*version\s*=' Cargo.toml | head -1 | sed 's/.*"\([^"]*\)".*/\1/')

if [ -z "$VERSION" ] || [ "$VERSION" = "version" ] || [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}Error: Could not extract valid version from Cargo.toml${NC}"
    echo -e "${RED}Found: '${VERSION}'${NC}"
    exit 1
fi

TAG_NAME="v${VERSION}"

echo -e "${GREEN}Found version in Cargo.toml: ${VERSION}${NC}"
echo -e "${GREEN}Tag name will be: ${TAG_NAME}${NC}"

# Fetch latest tags from remote
echo -e "\n${YELLOW}Fetching latest tags from remote...${NC}"
git fetch --tags --quiet

# Check if tag already exists (local or remote)
if git rev-parse "$TAG_NAME" >/dev/null 2>&1; then
    echo -e "${RED}Error: Tag ${TAG_NAME} already exists locally${NC}"
    exit 1
fi

if git ls-remote --tags origin "$TAG_NAME" | grep -q "$TAG_NAME"; then
    echo -e "${RED}Error: Tag ${TAG_NAME} already exists on remote${NC}"
    exit 1
fi

# Get all version tags and find the most recent one
# This handles tags in format v*.*.* or *.*.* (with or without v prefix) and sorts them by version
LATEST_TAG=$(git tag -l | grep -E '^(v)?[0-9]+\.[0-9]+\.[0-9]+$' | sort -V | tail -1)

if [ -z "$LATEST_TAG" ]; then
    echo -e "${YELLOW}No previous release tags found. This appears to be the first release.${NC}"
    LATEST_VERSION="none"
else
    # Extract version number from tag (remove 'v' prefix if present)
    LATEST_VERSION=$(echo "$LATEST_TAG" | sed 's/^v//')
    echo -e "${GREEN}Most recent release tag: ${LATEST_TAG}${NC}"
    
    # Compare versions - check if current version is greater than latest
    # Use sort -V to compare versions, if VERSION sorts before LATEST_VERSION, it's too old
    HIGHER_VERSION=$(printf '%s\n%s\n' "$VERSION" "$LATEST_VERSION" | sort -V | tail -1)
    
    if [ "$HIGHER_VERSION" != "$VERSION" ] || [ "$VERSION" = "$LATEST_VERSION" ]; then
        echo -e "\n${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${RED}Error: Version conflict${NC}"
        echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "Current version in Cargo.toml: ${YELLOW}${VERSION}${NC}"
        echo -e "Most recent release version:    ${YELLOW}${LATEST_VERSION}${NC}"
        echo -e ""
        echo -e "${RED}The version you're trying to release (${VERSION}) is not greater than the latest release (${LATEST_VERSION}).${NC}"
        echo -e ""
        
        # Suggest next version by incrementing patch version
        IFS='.' read -r -a LATEST_PARTS <<< "$LATEST_VERSION"
        MAJOR="${LATEST_PARTS[0]}"
        MINOR="${LATEST_PARTS[1]}"
        PATCH="${LATEST_PARTS[2]}"
        SUGGESTED_PATCH=$((PATCH + 1))
        SUGGESTED_VERSION="${MAJOR}.${MINOR}.${SUGGESTED_PATCH}"
        
        echo -e "${YELLOW}Suggested minimum version: ${GREEN}${SUGGESTED_VERSION}${NC}"
        echo -e "${YELLOW}Update Cargo.toml to version = \"${SUGGESTED_VERSION}\" and try again.${NC}"
        echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        exit 1
    fi
fi

# Confirmation prompt
echo -e "\n${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Release Confirmation${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "Most recent release version: ${GREEN}${LATEST_VERSION}${NC}"
echo -e "You are about to release:    ${GREEN}${VERSION}${NC}"
echo -e "Tag name:                     ${GREEN}${TAG_NAME}${NC}"
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "\n${YELLOW}Do you want to create and push tag ${TAG_NAME}? (y/N)${NC}"
read -r response

if [[ ! "$response" =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}Release cancelled.${NC}"
    exit 0
fi

# Create annotated tag
echo -e "\n${YELLOW}Creating annotated tag ${TAG_NAME}...${NC}"
git tag -a "$TAG_NAME" -m "Release ${TAG_NAME}"

# Push tag to remote
echo -e "${YELLOW}Pushing tag ${TAG_NAME} to remote...${NC}"
git push origin "$TAG_NAME"

echo -e "\n${GREEN}✓ Successfully created and pushed tag ${TAG_NAME}${NC}"
echo -e "${GREEN}GitHub Actions will now build and publish the release.${NC}"

