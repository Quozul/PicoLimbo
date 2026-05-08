package dev.quozul;

import com.sun.jna.Pointer;
import org.jetbrains.annotations.Nullable;

import java.nio.file.Path;

public class PicoLimboRunner implements Runnable {

    private final Path configurationPath;
    private final Standalone.RustLib lib;
    @Nullable
    private volatile Pointer cancellation_token;

    public PicoLimboRunner(Path configurationPath) throws Exception {
        lib = Standalone.loadLib();
        this.configurationPath = configurationPath;
    }

    @Override
    public void run() {
        String[] args = {
                "pico_limbo_java_wrapper",
                "--config",
                configurationPath.toString()
        };

        cancellation_token = lib.get_cancellation_token();
        try {
            lib.start_app(cancellation_token, args.length, args);
        } finally {
            lib.cleanup_token(cancellation_token);
            cancellation_token = null;
        }
    }

    public void stop() {
        lib.stop_app(cancellation_token);
    }
}
