# Performance Benchmarks

**Tested Version**: PicoLimbo v1.4.0+mc1.21.8

## Summary

PicoLimbo demonstrates exceptional performance efficiency in handling large numbers of concurrent Minecraft connections. In the benchmark test, **1,000 idle players** were successfully handled using **less than 1% CPU usage** and **less than 11 MB of RAM**. CPU usage peaked at 10% during the initial connection phase before stabilizing at minimal levels, showcasing PicoLimbo's ability to efficiently manage connection loads even on resource-constrained hardware.

## Testing Methodology

### Test Environment

**Server Hardware:**
- **CPU**: AMD Ryzen 9 7945HX (1 core allocated)
- **RAM**: 1 GB DDR5 (allocated limit)
- **Network**: 500 Mb/s
- **Platform**: Pterodactyl Panel (Docker containerized)

**Load Testing Client:**
- **CPU**: AMD Ryzen 9 5950X
- **RAM**: 32 GB DDR4
- **Tool**: [SoulFire MC](https://soulfiremc.com/) v2.0.1
- **Java Runtime**: OpenJDK 24.0.2

### Load Testing Configuration

```bash
java -Xmx4G -XX:+EnableDynamicAgentLoading \
     -XX:+UnlockExperimentalVMOptions -XX:+UseZGC -XX:+ZGenerational \
     -XX:+AlwaysActAsServerClassMachine -XX:+UseNUMA \
     -XX:+UseFastUnorderedTimeStamps -XX:+UseVectorCmov \
     -XX:+UseCriticalJavaThreadPriority -Dsf.flags.v1=true \
     -jar SoulFireCLI-2.0.1.jar \
     --bot-address limbo.quozul.dev:25565 \
     --bot-amount 1000 \
     --bot-protocol-version 1.21 \
     --bot-join-delay-min 10 \
     --bot-join-delay-max 30 \
     --start
```

### Test Parameters
- **Protocol Version**: Minecraft 1.21
- **Join Delay**: 10-30ms randomized intervals
- **Test Scenario**: Idle Connections (players connected but inactive)
- **Target Concurrent Users**: 1,000 players

## Results

### 1,000 Concurrent Idle Players

| Metric                      | Result  |
|-----------------------------|---------|
| **CPU Usage**               | < 1%    |
| **Memory Usage**            | < 11 MB |
| **Connection Success Rate** | 100%    |

### Key Findings

✅ **Exceptional Efficiency**: PicoLimbo can handle 1,000 concurrent connections using less than 1% CPU usage and less than 11 MB RAM once all players were connected

✅ **Connection Handling**: CPU usage peaked at ~10% during the connection phase, then dropped to minimal levels

✅ **Resource Constrained Performance**: Even with severe hardware limitations (1 CPU core, 1GB RAM), the server remained highly responsive

✅ **Containerized Deployment**: Excellent performance in Docker/Pterodactyl environment

✅ **Memory Stability**: Consistent memory usage with minimal fluctuation between connection and steady state phases
