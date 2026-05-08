package dev.quozul;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Platform;
import com.sun.jna.Pointer;
import org.jetbrains.annotations.Nullable;

import java.io.File;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.StandardCopyOption;

public class Standalone {

    public interface RustLib extends Library {

        // C: void start_app(CancellationToken* ptr, int argc, char** argv);
        void start_app(Pointer ptr, int argc, String[] argv);

        // C: void stop_app(CancellationToken* ptr);
        void stop_app(Pointer ptr);

        // C: CancellationToken* get_cancellation_token();
        @Nullable
        Pointer get_cancellation_token();

        // C: void cleanup_token(CancellationToken* ptr);
        void cleanup_token(Pointer ptr);
    }

    public static RustLib loadLib() throws Exception {
        String libName = BuildConstants.LIB_NAME;
        String resourcePath = getResourcePath(libName);

        String extension = resourcePath.substring(resourcePath.lastIndexOf('.'));
        File tempLib = File.createTempFile(libName, extension);
        tempLib.deleteOnExit();

        try (InputStream in = Standalone.class.getResourceAsStream(resourcePath)) {
            if (in == null) {
                throw new RuntimeException("Library file not found in JAR: " + resourcePath);
            }
            Files.copy(in, tempLib.toPath(), StandardCopyOption.REPLACE_EXISTING);
        }

        return Native.load(tempLib.getAbsolutePath(), RustLib.class);
    }

    public static void main(String[] args) {
        try {
            String[] effectiveArgs = new String[args.length + 1];
            effectiveArgs[0] = "pico_limbo_java_wrapper";
            System.arraycopy(args, 0, effectiveArgs, 1, args.length);

            RustLib lib = Standalone.loadLib();
            Pointer token = lib.get_cancellation_token();
            Runtime.getRuntime().addShutdownHook(new ShutdownHook(lib, token));
            lib.start_app(token, effectiveArgs.length, effectiveArgs);
        } catch (Exception e) {
            e.printStackTrace();
            System.exit(1);
        }
    }

    private static String getResourcePath(String libName) {
        String os = System.getProperty("os.name").toLowerCase();
        String arch = System.getProperty("os.arch").toLowerCase();

        if (arch.equals("amd64")) {
            arch = "x86_64";
        }

        String platformPath;
        String extension;
        String prefix = "lib";

        if (Platform.isWindows()) {
            platformPath = "windows/x86_64";
            extension = ".dll";
            prefix = "";
        } else if (Platform.isMac()) {
            if (arch.equals("aarch64")) {
                platformPath = "macos/aarch64";
                extension = ".dylib";
            } else {
                throw new UnsupportedOperationException("Unsupported macOS architecture: " + arch);
            }
        } else if (Platform.isLinux()) {
            if (arch.equals("x86_64")) {
                platformPath = "linux/x86_64";
            } else if (arch.equals("aarch64")) {
                platformPath = "linux/aarch64";
            } else {
                throw new UnsupportedOperationException("Unsupported Linux architecture: " + arch);
            }
            extension = ".so";
        } else {
            throw new UnsupportedOperationException("Unsupported OS: " + os);
        }

        return String.format("/%s/%s%s%s", platformPath, prefix, libName, extension);
    }

    private static class ShutdownHook extends Thread {
        private final RustLib lib;
        private final Pointer token;

        public ShutdownHook(RustLib lib, Pointer token) {
            this.lib = lib;
            this.token = token;
        }

        @Override
        public void run() {
            lib.stop_app(token);
        }
    }
}
