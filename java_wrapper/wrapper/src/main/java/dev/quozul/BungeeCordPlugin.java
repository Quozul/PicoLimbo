package dev.quozul;

import net.md_5.bungee.api.plugin.Plugin;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

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
        this.worker = new PicoLimboRunner(configurationFile);

        getProxy().getScheduler().runAsync(this, worker);
    }

    @Override
    public void onDisable() {
        if (worker != null)
            worker.stop();
    }
}
