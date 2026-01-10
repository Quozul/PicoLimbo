package dev.quozul;

import java.nio.file.Path;

public class PicoLimboRunner implements Runnable {

    private final Path configurationPath;

    public PicoLimboRunner(Path configurationPath) {
        this.configurationPath = configurationPath;
    }

    @Override
    public void run() {
        String[] args = {"--config", configurationPath.toString()};
        Standalone.main(args);
    }

    public void stop() {
    }
}
