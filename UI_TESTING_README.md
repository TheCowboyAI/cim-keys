# CIM Keys - UI Testing Documentation

Welcome to the comprehensive UI testing suite for cim-keys!

## 📚 Documentation Overview

This directory contains complete testing documentation for the cim-keys GUI application. Start here to understand the testing strategy and execution.

### Core Documents

| Document | Purpose | When to Use |
|----------|---------|-------------|
| **UI_TEST_SUMMARY.md** | Executive overview and quick reference | Start here - read first |
| **UI_TEST_PLAN.md** | Detailed 200+ test cases | Reference during testing |
| **UI_TEST_EXECUTION.md** | Step-by-step execution guide | Follow during manual testing |
| **test-ui-automated.sh** | Automated test script | Run before manual tests |

---

## 🚀 Quick Start

### 1. Read the Summary (5 minutes)
```bash
cat UI_TEST_SUMMARY.md
```
Understand:
- What works ✅
- What's pending ⏳
- What's broken ❌
- How to run tests

### 2. Run Automated Tests (2 minutes)
```bash
./test-ui-automated.sh
```
Verifies:
- Build succeeds
- Dependencies resolved
- Architecture correct
- Files present

### 3. Launch GUI Application
```bash
cargo run --features gui -- /tmp/cim-keys-ui-test
```
Or:
```bash
./target/debug/cim-keys-gui /tmp/cim-keys-ui-test
```

### 4. Follow Execution Guide
Open `UI_TEST_EXECUTION.md` and follow Phase 1-16 checklists.

---

## 📋 Test Document Details

### UI_TEST_SUMMARY.md
**Purpose**: High-level overview for decision makers and developers

**Contains**:
- Executive summary
- Quick stats (build status, test results)
- Feature completeness matrix
- Known issues with priority
- Performance expectations
- Architecture highlights
- Next steps for all stakeholders

**Best for**:
- Project managers
- New developers
- Stakeholders
- Quick status checks

---

### UI_TEST_PLAN.md
**Purpose**: Comprehensive test specification (200+ test cases)

**Contains**:
- 16 test categories
- Detailed test cases for each feature
- Expected behaviors
- Edge cases
- Integration scenarios
- Performance tests
- Accessibility tests

**Test Categories**:
1. Welcome Tab (14 tests)
2. Organization Tab (35 tests)
3. Keys Tab (52 tests)
4. Export Tab (24 tests)
5. Navigation (9 tests)
6. UI Theme & Styling (20 tests)
7. Animated Background (8 tests)
8. Error Handling (9 tests)
9. MVI Architecture (12 tests)
10. Offline & Air-Gapped (9 tests)
11. Data Integrity (9 tests)
12. Performance (12 tests)
13. Accessibility (9 tests)
14. Edge Cases (9 tests)
15. Integration (7 tests)
16. WASM Compatibility (6 tests)

**Best for**:
- QA engineers
- Comprehensive testing
- Reference documentation
- Test case design

---

### UI_TEST_EXECUTION.md
**Purpose**: Step-by-step manual testing guide

**Contains**:
- Pre-test setup instructions
- 16 detailed test phases
- Checkboxes for each test
- Expected results
- Actions to perform
- Verification steps
- Known issues inline

**Format**:
Each phase has:
```markdown
### Phase X: Feature Name ✅
- [ ] Test X.1: Specific test
  - Action: What to do
  - Expected: What should happen
```

**Best for**:
- Manual testers
- First-time testers
- Structured execution
- Progress tracking

---

### test-ui-automated.sh
**Purpose**: Automated verification before manual testing

**Tests**:
1. Build verification
2. Binary existence
3. Dependency resolution
4. Source file structure
5. Test data validation
6. Output directory creation
7. Module structure
8. Feature flags
9. Code quality (clippy)
10. Documentation presence
11. MVI architecture
12. Event-sourcing patterns

**Output**:
- ✓ Pass (green)
- ✗ Fail (red)
- ℹ Info (yellow)
- Summary statistics

**Best for**:
- Pre-testing validation
- CI/CD integration
- Quick health checks
- Automated regression

---

## 🎯 Testing Strategy

### Phase 1: Automated Checks (5 minutes)
Run `./test-ui-automated.sh`
- Ensures build works
- Verifies structure
- Checks architecture

### Phase 2: Core Functionality (30 minutes)
Focus on critical path:
1. Welcome → Create Domain
2. Organization → Add 3 people
3. Keys → Generate Root CA
4. Keys → Generate Intermediate CA
5. Keys → Generate Server Cert
6. Export → Export domain
7. Verify files created

### Phase 3: Comprehensive Testing (2-3 hours)
Follow `UI_TEST_EXECUTION.md` completely:
- All 16 phases
- All test cases
- All tabs and features
- Document all findings

### Phase 4: Exploratory Testing (1 hour)
Try to break things:
- Invalid inputs
- Rapid clicking
- Edge cases
- Unusual workflows

---

## ✅ Test Checklist

### Before Testing
- [ ] Read `UI_TEST_SUMMARY.md`
- [ ] Run `./test-ui-automated.sh` (must pass)
- [ ] Create test output directory
- [ ] Have bootstrap config ready
- [ ] Ensure graphical environment available

### During Testing
- [ ] Follow `UI_TEST_EXECUTION.md` phases
- [ ] Check off completed tests
- [ ] Document any issues found
- [ ] Take screenshots of problems
- [ ] Note unexpected behavior

### After Testing
- [ ] Update test documents with results
- [ ] File GitHub issues for bugs
- [ ] Update `UI_TEST_SUMMARY.md` with findings
- [ ] Report to development team

---

## 🐛 Reporting Issues

When you find a bug, document:

1. **Test Case**: Which test (e.g., "Phase 3, Test 3.2")
2. **Steps to Reproduce**:
   - Step 1: ...
   - Step 2: ...
3. **Expected Behavior**: What should happen
4. **Actual Behavior**: What actually happened
5. **Screenshots**: Visual evidence
6. **System Info**: OS, Rust version, etc.
7. **Severity**: Critical, High, Medium, Low

---

## 📊 Test Coverage Matrix

| Feature | Automated | Manual | Status |
|---------|-----------|--------|--------|
| Build & Setup | ✅ | ⏳ | Ready |
| Welcome Tab | ⏳ | ⏳ | Ready |
| Organization Tab | ⏳ | ⏳ | Ready |
| Keys Tab | ⏳ | ⏳ | Ready |
| Export Tab | ⏳ | ⏳ | Ready |
| Navigation | ⏳ | ⏳ | Ready |
| Theme/Styling | ⏳ | ⏳ | Ready |
| Animation | ⏳ | ⏳ | Ready |
| Error Handling | ⏳ | ⏳ | Ready |
| MVI Architecture | ✅ | ⏳ | Verified |
| Performance | ⏳ | ⏳ | Ready |

**Legend**:
- ✅ Tested and passing
- ⏳ Ready for testing
- ❌ Known issues
- 🚧 Not implemented

---

## 🔧 Test Environment

### Requirements
- **OS**: Linux (tested), macOS, Windows
- **Rust**: 1.70+ (edition 2021)
- **Display**: Graphical environment required
- **RAM**: 512 MB minimum
- **Disk**: 500 MB for output

### Optional Tools
- **jq**: JSON validation in automated tests
- **cargo-clippy**: Code quality checks
- **System monitor**: Track CPU/memory usage

---

## 📈 Success Metrics

### Definition of "Done"
- [ ] All automated tests pass
- [ ] All critical path tests pass
- [ ] No critical bugs found
- [ ] Performance meets expectations
- [ ] Documentation is accurate
- [ ] Known issues are documented

### Quality Gates
- **Build**: Must compile without errors
- **Unit Tests**: 100% pass rate
- **Core Features**: All functional
- **Performance**: <2s launch, 30 FPS animation
- **Usability**: No confusing UI elements

---

## 🤝 Contributing

Found an issue in the tests themselves?

1. Document the problem
2. Suggest improvement
3. Update relevant test document
4. Submit PR with explanation

---

## 📞 Support

### Questions About Testing
- Check `UI_TEST_SUMMARY.md` first
- Review `UI_TEST_PLAN.md` for details
- Consult `UI_TEST_EXECUTION.md` for steps

### Questions About Application
- See main `CLAUDE.md` in repository root
- Check architecture documentation
- Review code comments in `src/gui.rs`

### Reporting Results
Update `UI_TEST_SUMMARY.md` with:
- Date tested
- Tester name
- Results summary
- Issues found
- Recommendations

---

## 🎓 Testing Best Practices

### Do's ✅
- Read summary before starting
- Run automated tests first
- Follow execution guide in order
- Document everything
- Take screenshots
- Report issues promptly

### Don'ts ❌
- Skip automated tests
- Test out of order (first time)
- Assume issues are "already known"
- Forget to document findings
- Rush through tests
- Test without reading docs

---

## 📅 Test Schedule Recommendation

### Sprint 1: Validation (1 day)
- Run automated tests
- Execute core functionality tests
- Verify critical path works

### Sprint 2: Comprehensive (2-3 days)
- Complete all 16 phases
- Test all features
- Document all findings

### Sprint 3: Edge Cases (1 day)
- Exploratory testing
- Performance testing
- Stress testing

### Sprint 4: Regression (ongoing)
- Re-test after fixes
- Verify no new issues
- Update documentation

---

## 🏁 Getting Started Now

```bash
# 1. Read the summary (5 min)
less UI_TEST_SUMMARY.md

# 2. Run automated tests (2 min)
./test-ui-automated.sh

# 3. Launch GUI (immediate)
cargo run --features gui -- /tmp/cim-keys-ui-test

# 4. Open execution guide (in another terminal/window)
less UI_TEST_EXECUTION.md

# 5. Start testing!
# Follow Phase 1-16 checklists
```

---

**Happy Testing! 🎉**

This application represents a sophisticated event-sourced, offline-first key management system. Your testing helps ensure it's reliable, secure, and user-friendly for critical infrastructure deployment.

---

**Documentation Version**: 1.0
**Last Updated**: 2025-11-11
**Maintained By**: CIM Development Team
