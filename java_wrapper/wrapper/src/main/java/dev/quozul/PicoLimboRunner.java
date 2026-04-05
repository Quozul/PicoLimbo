package dev.quozul;

import java.nio.file.Path;

public class PicoLimboRunner implements Runnable {

    private final Path configurationPath;
    private volatile Standalone.RustLib lib;
    private volatile boolean running = false;

    public PicoLimboRunner(Path configurationPath) {
        this.configurationPath = configurationPath;
    }

    @Override
    public void run() {
        try {
            lib = Standalone.loadLib();
            running = true;

            String[] args = {
                    "pico_limbo_java_wrapper",
                    "--config",
                    configurationPath.toString()
            };

            lib.start_app(args.length, args);
        } catch (Exception e) {
            e.printStackTrace();
        } finally {
            running = false;
        }
    }

    public void stop() {
        if (lib != null && running)
            lib.stop_app();
    }
}
