# Performance Benchmarks

**Tested Version**: PicoLimbo v1.4.0+mc1.21.8

## Summary

PicoLimbo demonstrates exceptional performance efficiency across different architectures, handling large numbers of
concurrent Minecraft connections with minimal resource consumption. The benchmarks show that PicoLimbo can efficiently
manage **1,000 concurrent idle players** on both x86 and ARM platforms, using **less than 11 MB of RAM** and maintaining
low CPU usage once connections stabilize.

## Testing Methodology

### Load Testing Client

**Hardware:**

- **CPU**: AMD Ryzen 9 5950X
- **RAM**: 32 GB DDR4
- **Tool**: [SoulFire MC](https://soulfiremc.com/) v2.0.1
- **Java Runtime**: OpenJDK 24.0.2

**Configuration:**

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
- **Platform**: Pterodactyl Panel (Docker containerized)
- **Resource Limits**: 1 CPU core, 1GB RAM allocated

## Results

### x86 Performance (AMD Ryzen 9 7945HX)

**Server Environment:**

- **CPU**: AMD Ryzen 9 7945HX (1 core allocated)
- **RAM**: 1 GB DDR5 (allocated limit)
- **Network**: 500 Mb/s

| Metric           | Idle (0 Players) | Connection Phase   | Steady State     |
|------------------|------------------|--------------------|------------------|
| **CPU Usage**    | Flat 0.0%        | **Up to 10% peak** | **Less than 1%** |
| **Memory Usage** | More than 1 MB   | Less than 11 MB    | Less than 11 MB  |

### ARM Performance (Rockchip RK3588)

**Server Environment:**

- **CPU**: Rockchip RK3588 (1 core allocated)
- **RAM**: 1 GB LPDDR4 (allocated limit)
- **Network**: 500 Mb/s

| Metric           | Idle (0 Players)   | Connection Phase | Steady State    |
|------------------|--------------------|------------------|-----------------|
| **CPU Usage**    | Flat 0.0%          | Up to 48% peak   | Less than 8%    |
| **Memory Usage** | **Less than 1 MB** | Up to 11 MB      | Less than 11 MB |
