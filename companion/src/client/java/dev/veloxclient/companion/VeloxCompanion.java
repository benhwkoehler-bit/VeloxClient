package dev.veloxclient.companion;

import java.lang.management.ManagementFactory;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.fabricmc.fabric.api.client.screen.v1.ScreenEvents;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.screens.TitleScreen;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Reports how the game runs to the VeloxClient launcher. The launcher already
 * reads the game log live, so the reports are plain log lines with a fixed
 * prefix: {@code velox:fps=214} every few seconds and {@code velox:startup_ms=…}
 * once the title screen first appears. On the very first start it also shows
 * the {@link WelcomeScreen}.
 */
public final class VeloxCompanion implements ClientModInitializer {
	private static final Logger LOG = LoggerFactory.getLogger("VeloxCompanion");

	/** How often the FPS line is written; 20 client ticks are one second. */
	private static final int FPS_REPORT_TICKS = 5 * 20;

	private int ticksSinceReport;
	private boolean startupReported;

	@Override
	public void onInitializeClient() {
		ClientTickEvents.END_CLIENT_TICK.register(this::onClientTick);
		ScreenEvents.AFTER_INIT.register((client, screen, width, height) -> {
			if (!(screen instanceof TitleScreen)) {
				return;
			}
			if (!startupReported) {
				startupReported = true;
				long startupMs = System.currentTimeMillis() - ManagementFactory.getRuntimeMXBean().getStartTime();
				LOG.info("velox:startup_ms={}", startupMs);
			}
			if (!WelcomeState.seen()) {
				// Switch on the next frame, not while the title screen is still initialising
				client.execute(() -> client.setScreenAndShow(new WelcomeScreen(screen)));
			}
		});
	}

	private void onClientTick(Minecraft client) {
		if (++ticksSinceReport < FPS_REPORT_TICKS) {
			return;
		}
		ticksSinceReport = 0;
		LOG.info("velox:fps={}", client.getFps());
	}
}
