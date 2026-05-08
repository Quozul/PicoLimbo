package dev.quozul;

import net.md_5.bungee.api.plugin.Plugin;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.logging.Level;

public class BungeeCordPlugin extends Plugin {

    private PicoLimboRunner worker;

    @Override
    public void onEnable() {
        Path dataDirectory = getDataFolder().toPath();
        if (Files.exists(dataDirectory)) {
            try {
                Files.createDirectories(dataDirectory);
            } catch (IOException e) {
                getLogger().info("Error creating data directory");
                return;
            }
        }

        Path configurationFile = dataDirectory.resolve("server.toml");
        try {
            worker = new PicoLimboRunner(configurationFile);
            getProxy().getScheduler().runAsync(this, worker);
        } catch (Exception e) {
            getLogger().log(Level.WARNING, "Error initializing PicoLimbo", e);
        }
    }

    @Override
    public void onDisable() {
        if (worker != null) {
            worker.stop();
        }
    }
}
