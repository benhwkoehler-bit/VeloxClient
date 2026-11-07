package dev.veloxclient.companion;

import java.util.List;

import net.fabricmc.loader.api.FabricLoader;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.screens.Screen;
import net.minecraft.network.chat.Component;

/**
 * Shown once, before the title screen of the very first start: a terminal
 * window opens, the VELOX logo is drawn pixel by pixel, a few lines type
 * themselves and "Welcome to VeloxClient" is typed out large with a blinking
 * cursor. Continue or Esc moves on to the title screen.
 */
public final class WelcomeScreen extends Screen {
	// Velox colours (ARGB)
	private static final int PAGE = 0xFF0B0B10;
	private static final int PANEL = 0xFF12111A;
	private static final int BORDER = 0xFF262433;
	private static final int TEXT = 0xFFECEAF4;
	private static final int DIM = 0xFF8C88A0;
	private static final int ACCENT = 0xFFA78BFA;
	private static final int ACCENT_STRONG = 0xFF7C3AED;

	// Timeline in milliseconds
	private static final long LOGO_END = 1400;
	private static final long LINES_END = 3000;
	private static final long WELCOME_END = 4600;
	private static final long CURSOR_BLINK = 530;

	/** VELOX in a 5 x 5 pixel font, same as the launcher's first start. */
	private static final String[] LOGO = {
		"X...X XXXXX X.... .XXX. X...X",
		"X...X X.... X.... X...X .X.X.",
		"X...X XXXX. X.... X...X ..X..",
		".X.X. X.... X.... X...X .X.X.",
		"..X.. XXXXX XXXXX .XXX. X...X",
	};
	private static final int PIXEL = 4;
	private static final int PIXEL_GAP = 1;

	private static final int PANEL_WIDTH = 300;
	private static final int PANEL_HEIGHT = 170;
	private static final int LINE_HEIGHT = 11;
	private static final int WELCOME_SCALE = 2;

	private final Screen next;
	private final List<String[]> bootLines;
	private final long openedAt = System.currentTimeMillis();
	private Button continueButton;

	public WelcomeScreen(Screen next) {
		super(Component.literal("Welcome to VeloxClient"));
		this.next = next;
		this.bootLines = List.of(
			new String[] {"velox ❯ ", "./start"},
			new String[] {"[ ok ] ", "renderer ....... " + rendererName()},
			new String[] {"[ ok ] ", "mods ........... " + modCount()},
			new String[] {"[ ok ] ", "velox pack ..... ready"}
		);
	}

	@Override
	protected void init() {
		continueButton = addRenderableWidget(
			new Button.Builder(Component.literal("Continue"), button -> onClose())
				.bounds(width / 2 - 60, panelTop() + PANEL_HEIGHT + 12, 120, 20)
				.build()
		);
		continueButton.visible = false;
	}

	@Override
	public void extractBackground(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float delta) {
		graphics.fill(0, 0, width, height, PAGE);
	}

	@Override
	public void extractRenderState(GuiGraphicsExtractor graphics, int mouseX, int mouseY, float delta) {
		super.extractRenderState(graphics, mouseX, mouseY, delta);
		long elapsed = System.currentTimeMillis() - openedAt;

		int left = width / 2 - PANEL_WIDTH / 2;
		int top = panelTop();
		drawPanel(graphics, left, top);

		int x = left + 12;
		int y = top + 22;
		drawLogo(graphics, x, y, phase(elapsed, 0, LOGO_END));
		y += LOGO.length * (PIXEL + PIXEL_GAP) + 8;

		y = drawBootLines(graphics, x, y, elapsed);
		y += 6;
		drawWelcome(graphics, x, y, elapsed);

		boolean done = elapsed >= WELCOME_END;
		continueButton.visible = done;
		if (done) {
			graphics.text(font, "# your launcher shows live FPS while you play", x, top + PANEL_HEIGHT - 14, DIM, false);
		}
	}

	private void drawPanel(GuiGraphicsExtractor graphics, int left, int top) {
		int right = left + PANEL_WIDTH;
		int bottom = top + PANEL_HEIGHT;
		graphics.fill(left, top, right, bottom, PANEL);
		graphics.fill(left, top, right, top + 1, BORDER);
		graphics.fill(left, bottom - 1, right, bottom, BORDER);
		graphics.fill(left, top, left + 1, bottom, BORDER);
		graphics.fill(right - 1, top, right, bottom, BORDER);
		graphics.fill(left, top + 14, right, top + 15, BORDER);
		graphics.text(font, "velox@client: ~/start", left + 6, top + 4, DIM, false);
	}

	/** Column by column; the newest column flashes white before turning purple. */
	private void drawLogo(GuiGraphicsExtractor graphics, int x, int y, float drawn) {
		int columns = LOGO[0].length();
		int visible = (int) Math.ceil(drawn * columns);
		for (int row = 0; row < LOGO.length; row++) {
			for (int column = 0; column < Math.min(visible, columns); column++) {
				if (LOGO[row].charAt(column) != 'X') {
					continue;
				}
				int color = column + 1 == visible && drawn < 1f ? TEXT : ACCENT_STRONG;
				int px = x + column * (PIXEL + PIXEL_GAP);
				int py = y + row * (PIXEL + PIXEL_GAP);
				graphics.fill(px, py, px + PIXEL, py + PIXEL, color);
			}
		}
	}

	private int drawBootLines(GuiGraphicsExtractor graphics, int x, int y, long elapsed) {
		long share = (LINES_END - LOGO_END) / bootLines.size();
		for (int i = 0; i < bootLines.size(); i++) {
			long start = LOGO_END + share * i;
			if (elapsed < start) {
				break;
			}
			String[] line = bootLines.get(i);
			String shown = typed(line[1], phase(elapsed, start, start + share));
			int prefixColor = i == 0 ? ACCENT : DIM;
			graphics.text(font, line[0], x, y, prefixColor, false);
			graphics.text(font, shown, x + font.width(line[0]), y, TEXT, false);
			y += LINE_HEIGHT;
		}
		return y;
	}

	private void drawWelcome(GuiGraphicsExtractor graphics, int x, int y, long elapsed) {
		if (elapsed < LINES_END) {
			return;
		}
		String shown = typed("Welcome to VeloxClient", phase(elapsed, LINES_END, WELCOME_END));
		boolean typing = elapsed < WELCOME_END;
		boolean cursorOn = typing || (elapsed / CURSOR_BLINK) % 2 == 0;

		graphics.pose().pushMatrix();
		graphics.pose().translate(x, y);
		graphics.pose().scale(WELCOME_SCALE, WELCOME_SCALE);
		graphics.text(font, shown, 0, 0, TEXT, false);
		if (cursorOn) {
			int cx = font.width(shown) + 1;
			graphics.fill(cx, -1, cx + 5, font.lineHeight, ACCENT_STRONG);
		}
		graphics.pose().popMatrix();
	}

	@Override
	public boolean shouldCloseOnEsc() {
		return true;
	}

	@Override
	public void onClose() {
		WelcomeState.markSeen();
		minecraft.setScreenAndShow(next);
	}

	private int panelTop() {
		return Math.max(10, height / 2 - PANEL_HEIGHT / 2 - 16);
	}

	/** 0 before {@code start}, 1 after {@code end}, linear in between. */
	static float phase(long elapsed, long start, long end) {
		if (elapsed <= start) {
			return 0f;
		}
		if (elapsed >= end) {
			return 1f;
		}
		return (float) (elapsed - start) / (end - start);
	}

	/** The first {@code share} of {@code text}, by code points. */
	static String typed(String text, float share) {
		int count = text.codePointCount(0, text.length());
		int shown = Math.min(count, Math.round(share * count));
		return text.substring(0, text.offsetByCodePoints(0, shown));
	}

	private static String rendererName() {
		FabricLoader loader = FabricLoader.getInstance();
		if (loader.isModLoaded("puremetal")) {
			return "pure metal";
		}
		return loader.isModLoaded("sodium") ? "sodium" : "standard";
	}

	private static long modCount() {
		return FabricLoader.getInstance().getAllMods().stream()
			.map(mod -> mod.getMetadata().getId())
			.filter(id -> !id.startsWith("fabric-") && !id.equals("java") && !id.equals("minecraft")
				&& !id.equals("fabricloader") && !id.equals("mixinextras"))
			.count();
	}
}
