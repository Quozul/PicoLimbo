package dev.quozul;

import com.google.inject.Inject;
import com.velocitypowered.api.event.Subscribe;
import com.velocitypowered.api.event.proxy.ProxyInitializeEvent;
import com.velocitypowered.api.event.proxy.ProxyShutdownEvent;
import com.velocitypowered.api.plugin.Plugin;
import com.velocitypowered.api.plugin.annotation.DataDirectory;
import com.velocitypowered.api.proxy.ProxyServer;
import com.velocitypowered.api.scheduler.ScheduledTask;
import org.slf4j.Logger;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

@Plugin(
        id = "pico_limbo_java_wrapper",
        name = "Velocity plugin to run PicoLimbo inside of Velocity",
        version = "1.0",
        authors = {"Quozul"}
)
public class VelocityPlugin {

    private final ProxyServer server;
    private final Logger logger;
    private final Object plugin;
    private final Path dataDirectory;
    private PicoLimboRunner worker;
    private ScheduledTask task;

    @Inject
    public VelocityPlugin(ProxyServer server, Logger logger, @DataDirectory Path dataDirectory) {
        this.server = server;
        this.logger = logger;
        this.plugin = this;
        this.dataDirectory = dataDirectory;
    }

    @Subscribe
    public void onProxyInitialization(ProxyInitializeEvent event) {
        if (!Files.exists(dataDirectory)) {
            try {
                Files.createDirectories(dataDirectory);
            } catch (IOException e) {
                logger.error("Error creating data directory", e);
                return;
            }
        }

        Path configurationFile = dataDirectory.resolve("server.toml");
        this.worker = new PicoLimboRunner(configurationFile);

        this.task = server.getScheduler()
                .buildTask(plugin, worker)
                .schedule();
    }

    @Subscribe
    public void onProxyShutdown(ProxyShutdownEvent event) {
        if (worker != null) {
            worker.stop();
        }

        if (task != null) {
            task.cancel();
        }
    }
}
