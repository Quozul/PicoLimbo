package dev.quozul;

import com.sun.jna.Pointer;

import java.nio.file.Path;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicBoolean;

/**
 * Runs one independent PicoLimbo server instance.
 *
 * Each instance owns its cancellation token and can be started/stopped without
 * affecting other instances in the same JVM.
 */
public final class PicoLimboRunner implements Runnable, AutoCloseable {

    private final String instanceName;
    private final Path configurationPath;
    private final String[] additionalArguments;
    private final Standalone.RustLib lib;
    private final Object lifecycleLock = new Object();
    private final AtomicBoolean started = new AtomicBoolean();
    private final CompletableFuture<Void> completion = new CompletableFuture<>();

    private volatile Pointer cancellationToken;
    private volatile Thread workerThread;
    private boolean stopRequested;

    public PicoLimboRunner(Path configurationPath) throws Exception {
        this("picolimbo", configurationPath);
    }

    public PicoLimboRunner(String instanceName, Path configurationPath, String... additionalArguments)
            throws Exception {
        this.instanceName = Objects.requireNonNull(instanceName, "instanceName");
        this.configurationPath = Objects.requireNonNull(configurationPath, "configurationPath");
        this.additionalArguments = additionalArguments == null
                ? new String[0]
                : additionalArguments.clone();
        this.lib = Standalone.loadLib();
    }

    /**
     * Starts this server on a dedicated virtual thread.
     */
    public void start() {
        synchronized (lifecycleLock) {
            if (stopRequested) {
                throw new IllegalStateException("PicoLimbo instance has already been stopped: " + instanceName);
            }
            if (!started.compareAndSet(false, true)) {
                throw new IllegalStateException("PicoLimbo instance has already been started: " + instanceName);
            }

            Thread thread = Thread.ofVirtual()
                    .name("picolimbo-" + instanceName)
                    .unstarted(this::runServer);
            workerThread = thread;
            thread.start();
        }
    }

    /**
     * Runnable entry point for proxy schedulers that manage their own worker threads.
     */
    @Override
    public void run() {
        if (!started.compareAndSet(false, true)) {
            return;
        }
        workerThread = Thread.currentThread();
        runServer();
    }

    private void runServer() {
        Pointer token = null;
        try {
            String[] args = new String[3 + additionalArguments.length];
            args[0] = "pico_limbo_java_wrapper";
            args[1] = "--config";
            args[2] = configurationPath.toString();
            System.arraycopy(additionalArguments, 0, args, 3, additionalArguments.length);

            token = lib.get_cancellation_token();
            if (token == null) {
                throw new IllegalStateException("Native PicoLimbo failed to allocate a cancellation token");
            }

            synchronized (lifecycleLock) {
                cancellationToken = token;
                if (stopRequested) {
                    lib.stop_app(token);
                }
            }

            int exitCode = lib.start_app(token, args.length, args);
            if (exitCode == 0) {
                completion.complete(null);
            } else {
                completion.completeExceptionally(new IllegalStateException(
                        "PicoLimbo instance '" + instanceName + "' exited with code " + exitCode));
            }
        } catch (Throwable error) {
            completion.completeExceptionally(error);
        } finally {
            if (token != null) {
                synchronized (lifecycleLock) {
                    cancellationToken = null;
                }
                lib.cleanup_token(token);
            }
        }
    }

    /**
     * Requests graceful shutdown. Safe to call before, during, or after startup.
     */
    public void stop() {
        synchronized (lifecycleLock) {
            stopRequested = true;
            if (cancellationToken != null) {
                lib.stop_app(cancellationToken);
            }
        }
    }

    public boolean isRunning() {
        Thread thread = workerThread;
        return thread != null && thread.isAlive();
    }

    /**
     * Completes when the native server exits. Exceptional completion reports a JNA/runtime failure.
     */
    public CompletableFuture<Void> completion() {
        return completion;
    }

    public void await() throws InterruptedException {
        Thread thread = workerThread;
        if (thread != null) {
            thread.join();
        }
    }

    @Override
    public void close() {
        stop();
    }
}
