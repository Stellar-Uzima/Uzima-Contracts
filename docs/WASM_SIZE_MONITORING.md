# WASM Size Monitoring System

## Overview

This document describes the WASM size monitoring system implemented for Stellar smart contracts to prevent deployment failures due to size limitations.

## Stellar Limits

- **Max contract size**: 64KB (65,536 bytes)
- **Max transaction size**: 64KB (65,536 bytes)
- **Warning threshold**: 80% (51.2KB)
- **Critical threshold**: 95% (61.4KB)

## Features

### 1. Real-time Size Monitoring
- Tracks WASM file sizes for all contracts
- Provides color-coded status indicators
- Shows percentage of Stellar limit usage

### 2. Trend Analysis
- Maintains historical size data
- Shows size trends over builds
- Identifies growth patterns

### 3. Optimization Recommendations
- Provides targeted optimization tips based on size ranges
- Suggests specific actions for different size categories
- Links to helpful tools and resources

### 4. CI Integration
- Automated size checks in CI pipeline
- Fails builds on critical size violations
- Uploads trend data as artifacts

## Usage

### Command Line

#### Full Monitoring with Trends
```bash
# Build contracts and monitor sizes
make monitor-wasm

# Or run the script directly
./scripts/wasm_size_monitor.sh
```

#### Quick Size Check
```bash
# Fast size check without trend analysis
make check-wasm-size
```

#### Export Trend Data
```bash
# Export trend data as JSON
./scripts/wasm_size_monitor.sh --export-trends
```

### CI Pipeline

The monitoring system is integrated into the CI pipeline:

```yaml
- name: Monitor WASM sizes
  run: |
    echo "=== WASM Size Monitoring ==="
    ./scripts/wasm_size_monitor.sh
```

## Output Format

### Status Indicators

- **GREEN** (`OK`): Contract within safe limits (< 80%)
- **YELLOW** (`WARNING`): Contract approaching limit (80-95%)
- **RED** (`CRITICAL`): Contract exceeds safe limit (> 95%)

### Example Output

```
=== WASM Size Monitor ===
Stellar contract size limit: 64KB
Warning threshold: 80%
Critical threshold: 95%

=== Contract Size Analysis ===
Contract                  Size   Usage Status
--------                  ----   ----- ------
aml_contract              45KB   69.5% WARNING
  Trend: +2.3% over last 5 builds
  Optimization recommendations:
    - Review error message sizes
    - Use more efficient data structures
    - Remove debug code and assertions
    - Optimize serialization formats

audit_contract            28KB   43.2% OK
  Trend: Stable over last 5 builds

=== WASM Size Summary ===
Total contracts: 2
Total size: 73KB
Average size: 36KB
Warning contracts: 1

=== Recommendations ===
1. Run 'cargo install cargo-bloat' to analyze large functions
2. Use 'cargo build --target wasm32-unknown-unknown --release' for optimized builds
3. Consider splitting large contracts into smaller ones
4. Review dependencies and remove unused ones
```

## Trend Tracking

The system maintains trend data in `.wasm_size_trends.json`:

```json
{
  "contracts": {
    "aml_contract": {
      "size_history": [
        {"size": 44236, "timestamp": "2024-01-15T10:30:00Z"},
        {"size": 45012, "timestamp": "2024-01-15T11:15:00Z"}
      ],
      "current_size": 45012,
      "last_updated": "2024-01-15T11:15:00Z"
    }
  },
  "last_updated": "2024-01-15T11:15:00Z"
}
```

## Optimization Strategies

### For 50-60KB Contracts
- Remove unused dependencies and features
- Use cargo-bloat to identify large functions
- Consider splitting contract into multiple contracts
- Optimize string operations and reduce allocations

### For 40-50KB Contracts
- Review error message sizes
- Use more efficient data structures
- Remove debug code and assertions
- Optimize serialization formats

### For 30-40KB Contracts
- Use feature flags for optional functionality
- Review and optimize imports
- Consider using no_std when possible
- Optimize enum representations

## Tools and Utilities

### cargo-bloat
Install and use cargo-bloat to analyze what's contributing to WASM size:

```bash
cargo install cargo-bloat
cargo bloat --target wasm32-unknown-unknown --release --crates
```

### wasm-opt
Use wasm-opt from Binaryen for additional optimizations:

```bash
# Install Binaryen
# macOS: brew install binaryen
# Ubuntu: sudo apt-get install binaryen

# Optimize WASM
wasm-opt -Oz input.wasm -o output.wasm
```

### Size Analysis Commands
```bash
# Check individual contract size
wc -c dist/contract_name.wasm

# Find largest contracts
ls -lh dist/*.wasm | sort -k5 -hr

# Analyze WASM sections
wasm-objdump -h dist/contract_name.wasm
```

## Configuration

### Thresholds
Modify thresholds in `scripts/wasm_size_monitor.sh`:

```bash
MAX_CONTRACT_SIZE=65536      # 64KB Stellar limit
WARNING_THRESHOLD=0.8        # 80% warning threshold
CRITICAL_THRESHOLD=0.95     # 95% critical threshold
```

### Optimization Tips
Customize optimization recommendations in `.wasm_optimization_tips.json`.

## Troubleshooting

### Common Issues

1. **Missing Dependencies**
   ```bash
   # Install jq and bc
   sudo apt-get install jq bc  # Ubuntu/Debian
   brew install jq bc          # macOS
   ```

2. **No WASM Files Found**
   ```bash
   # Build contracts first
   make dist
   ```

3. **Permission Errors**
   ```bash
   # Make script executable
   chmod +x scripts/wasm_size_monitor.sh
   ```

### Debug Mode
Run with debug output:

```bash
bash -x scripts/wasm_size_monitor.sh
```

## Integration Examples

### Pre-commit Hook
```bash
#!/bin/sh
# .git/hooks/pre-commit
make check-wasm-size
```

### GitHub Actions
```yaml
- name: Check WASM Sizes
  run: |
    make dist
    make monitor-wasm
```

### Docker Integration
```dockerfile
RUN apt-get update && apt-get install -y jq bc
COPY scripts/wasm_size_monitor.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/wasm_size_monitor.sh
```

## Best Practices

1. **Monitor Early**: Check sizes frequently during development
2. **Trend Analysis**: Watch for size creep over time
3. **Optimization**: Address size issues before they become critical
4. **Documentation**: Keep size optimization decisions documented
5. **Testing**: Verify contracts still work after optimization

## Performance Impact

- **Runtime**: Minimal overhead (< 1 second for typical projects)
- **Storage**: Trend data typically < 10KB for most projects
- **CI Impact**: Adds ~30 seconds to build time

## Security Considerations

- Trend data files should not contain sensitive information
- WASM files are analyzed locally, no external services used
- No network connectivity required for basic monitoring

## Support

For issues with the WASM monitoring system:

1. Check this documentation first
2. Verify all dependencies are installed
3. Ensure WASM files are built with `make dist`
4. Check script permissions on Unix systems

## Future Enhancements

Potential improvements for the monitoring system:

- Graphical trend visualization
- Integration with code analysis tools
- Automated optimization suggestions
- Size impact prediction for code changes
- Integration with IDE for real-time feedback
