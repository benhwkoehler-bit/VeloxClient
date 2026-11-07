package dev.veloxclient.companion;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

import net.fabricmc.loader.api.FabricLoader;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/** Remembers that the welcome screen was shown, with a marker file in the config folder. */
final class WelcomeState {
	private static final Logger LOG = LoggerFactory.getLogger("VeloxCompanion");
	private static final String MARKER = "velox-companion-welcome-seen";

	private WelcomeState() {
	}

	private static Path marker() {
		return FabricLoader.getInstance().getConfigDir().resolve(MARKER);
	}

	static boolean seen() {
		return Files.exists(marker());
	}

	static void markSeen() {
		try {
			Files.createDirectories(marker().getParent());
			Files.writeString(marker(), "shown\n");
		} catch (IOException err) {
			// Not fatal: the welcome screen would just show once more next start
			LOG.warn("could not remember that the welcome screen was shown", err);
		}
	}
}
