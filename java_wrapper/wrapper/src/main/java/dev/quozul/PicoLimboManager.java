package dev.quozul;

import java.nio.file.Path;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ConcurrentMap;

/**
 * Manages multiple independent PicoLimbo instances inside one JVM.
 */
public final class PicoLimboManager implements AutoCloseable {

    private final ConcurrentMap<String, PicoLimboRunner> instances = new ConcurrentHashMap<>();

    /**
     * Creates an instance without starting it. The returned runner can be scheduled by a proxy.
     */
    public PicoLimboRunner create(String name, Path configurationPath, String... additionalArguments)
            throws Exception {
        Objects.requireNonNull(name, "name");
        if (name.isBlank()) {
            throw new IllegalArgumentException("Instance name must not be blank");
        }

        PicoLimboRunner runner = new PicoLimboRunner(name, configurationPath, additionalArguments);
        if (instances.putIfAbsent(name, runner) != null) {
            throw new IllegalStateException("PicoLimbo instance already exists: " + name);
        }
        return runner;
    }

    /**
     * Creates and starts an instance on its own virtual thread.
     */
    public PicoLimboRunner start(String name, Path configurationPath, String... additionalArguments)
            throws Exception {
        PicoLimboRunner runner = create(name, configurationPath, additionalArguments);
        try {
            runner.start();
            return runner;
        } catch (RuntimeException error) {
            instances.remove(name, runner);
            throw error;
        }
    }

    public Optional<PicoLimboRunner> get(String name) {
        return Optional.ofNullable(instances.get(name));
    }

    /**
     * Requests shutdown and removes the instance from this manager.
     */
    public boolean stop(String name) {
        PicoLimboRunner runner = instances.remove(name);
        if (runner == null) {
            return false;
        }
        runner.stop();
        return true;
    }

    public Map<String, PicoLimboRunner> instances() {
        return Map.copyOf(instances);
    }

    @Override
    public void close() {
        instances.forEach((name, runner) -> runner.stop());
        instances.clear();
    }
}
