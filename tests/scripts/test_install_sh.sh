#!/usr/bin/env bash
# Test script for install.sh
# Verifies that the bash installation script works correctly

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "\n${CYAN}=== Testing install.sh ===${NC}"

# Test 1: Script file exists
echo -e "\n${YELLOW}Test 1: Checking if install.sh exists...${NC}"
if [ -f "scripts/install.sh" ]; then
    echo -e "${GREEN}✓ PASS: install.sh exists${NC}"
else
    echo -e "${RED}✗ FAIL: install.sh not found${NC}"
    exit 1
fi

# Test 2: Script has proper bash shebang
echo -e "\n${YELLOW}Test 2: Checking for bash shebang...${NC}"
if head -n 1 scripts/install.sh | grep -q "#!/.*bash"; then
    echo -e "${GREEN}✓ PASS: Valid bash shebang${NC}"
else
    echo -e "${RED}✗ FAIL: Missing or invalid bash shebang${NC}"
    exit 1
fi

# Test 3: Script has set -e for error handling
echo -e "\n${YELLOW}Test 3: Checking for error handling (set -e)...${NC}"
if grep -q "set -e" scripts/install.sh; then
    echo -e "${GREEN}✓ PASS: Error handling present${NC}"
else
    echo -e "${RED}✗ FAIL: Error handling missing${NC}"
    exit 1
fi

# Test 4: Script has OS detection
echo -e "\n${YELLOW}Test 4: Checking for OS detection...${NC}"
if grep -q "uname -s" scripts/install.sh; then
    echo -e "${GREEN}✓ PASS: OS detection present${NC}"
else
    echo -e "${RED}✗ FAIL: OS detection missing${NC}"
    exit 1
fi

# Test 5: Script has architecture detection
echo -e "\n${YELLOW}Test 5: Checking for architecture detection...${NC}"
if grep -q "uname -m" scripts/install.sh; then
    echo -e "${GREEN}✓ PASS: Architecture detection present${NC}"
else
    echo -e "${RED}✗ FAIL: Architecture detection missing${NC}"
    exit 1
fi

# Test 6: Script has GitHub API integration
echo -e "\n${YELLOW}Test 6: Checking for GitHub API integration...${NC}"
if grep -q "api.github.com" scripts/install.sh; then
    echo -e "${GREEN}✓ PASS: GitHub API integration present${NC}"
else
    echo -e "${RED}✗ FAIL: GitHub API integration missing${NC}"
    exit 1
fi

# Test 7: Script has dependency checking
echo -e "\n${YELLOW}Test 7: Checking for dependency checking...${NC}"
if grep -q "command -v" scripts/install.sh; then
    echo -e "${GREEN}✓ PASS: Dependency checking present${NC}"
else
    echo -e "${RED}✗ FAIL: Dependency checking missing${NC}"
    exit 1
fi

# Test 8: Script has PATH checking
echo -e "\n${YELLOW}Test 8: Checking for PATH verification...${NC}"
if grep -q "PATH" scripts/install.sh; then
    echo -e "${GREEN}✓ PASS: PATH verification present${NC}"
else
    echo -e "${RED}✗ FAIL: PATH verification missing${NC}"
    exit 1
fi

# Test 9: Script has installation verification
echo -e "\n${YELLOW}Test 9: Checking for installation verification...${NC}"
if grep -q -- "--version" scripts/install.sh; then
    echo -e "${GREEN}✓ PASS: Installation verification present${NC}"
else
    echo -e "${RED}✗ FAIL: Installation verification missing${NC}"
    exit 1
fi

# Test 10: Script has color output
echo -e "\n${YELLOW}Test 10: Checking for color output...${NC}"
if grep -q "033\[" scripts/install.sh; then
    echo -e "${GREEN}✓ PASS: Color output present${NC}"
else
    echo -e "${RED}✗ FAIL: Color output missing${NC}"
    exit 1
fi

echo -e "\n${GREEN}=== All tests passed! ===${NC}"
exit 0
