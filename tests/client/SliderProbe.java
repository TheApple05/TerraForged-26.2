import com.google.gson.Gson;
import com.mojang.blaze3d.platform.InputConstants;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import net.minecraft.SharedConstants;
import net.minecraft.client.input.KeyEvent;
import net.minecraft.client.input.MouseButtonEvent;
import net.minecraft.client.input.MouseButtonInfo;
import net.minecraft.network.chat.Component;
import net.minecraft.server.Bootstrap;
import raccoonman.reterraforged.client.gui.widget.Slider;

public final class SliderProbe {
    private static int assertions;
    private static void check(boolean condition, String message) {
        assertions++;
        if (!condition) throw new AssertionError(message);
    }
    private static void equal(double actual, double expected, String message) {
        check(Double.doubleToRawLongBits(actual) == Double.doubleToRawLongBits(expected),
              message + ": expected " + expected + ", got " + actual);
    }
    private static MouseButtonEvent mouse(double x) {
        return new MouseButtonEvent(x, 25, new MouseButtonInfo(0, 0));
    }

    public static void main(String[] args) {
        Locale.setDefault(Locale.ROOT);
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        check(SharedConstants.getCurrentVersion().id().equals("26.2"), "Wrong game version");
        List<Double> updates = new ArrayList<>();
        double[] setting = {25};
        Slider slider = new Slider(10, 20, 108, 20, 25, 0, 100, Component.literal("Setting"),
            Slider.Format.FLOAT, (control, value) -> {
                updates.add(value);
                setting[0] = control.scaleValue(value);
                return value;
            });
        check(updates.isEmpty(), "Construction must not trigger user callbacks");
        equal(slider.getLerpedValue(), 25, "Initial setting");

        // Real 26.2 onClick -> private setValueFromMouse -> virtual setValue path.
        slider.onClick(mouse(64), false);
        check(updates.size() == 1, "Mouse click must invoke the settings callback");
        equal(setting[0], 50, "Mouse click changed model setting");
        equal(slider.getValue(), .5, "Mouse click normalized value");
        check(slider.getMessage().getString().contains("50.000"), "Mouse click must update displayed value");

        check(slider.mouseDragged(mouse(89), 25, 0), "Actual drag event must be consumed");
        check(updates.size() == 2, "Drag must invoke callback");
        equal(setting[0], 75, "Drag changed model setting");
        check(slider.getMessage().getString().contains("75.000"), "Drag updated label");

        check(slider.keyPressed(new KeyEvent(InputConstants.KEY_SPACE, 0, 0)), "Keyboard edit mode toggle");
        check(slider.keyPressed(new KeyEvent(InputConstants.KEY_RIGHT, 0, 0)), "Right-arrow input");
        check(updates.size() == 3, "Right-arrow event must invoke callback");
        double expectedRight = .75 + (double) (1.0F / 100);
        equal(slider.getValue(), expectedRight, "Vanilla right-arrow arithmetic");
        equal(setting[0], expectedRight * 100, "Keyboard changed model setting");
        check(slider.getMessage().getString().contains("76.000"), "Keyboard updated label");
        check(slider.keyPressed(new KeyEvent(InputConstants.KEY_LEFT, 0, 0)), "Left-arrow input");
        check(updates.size() == 4, "Left-arrow event must invoke callback");
        equal(slider.getValue(), .75, "Left-arrow returned to prior value");

        slider.setValue(2);
        equal(slider.getValue(), 1, "Setter must retain vanilla upper clamp");
        equal(setting[0], 100, "Upper clamp propagated to model");
        slider.setValue(-2);
        equal(slider.getValue(), 0, "Setter must retain vanilla lower clamp");
        equal(setting[0], 0, "Lower clamp propagated to model");
        int before = updates.size();
        slider.setValue(0);
        check(updates.size() == before, "Unchanged value must not fire an extra callback");

        Slider corrected = new Slider(0, 0, 108, 20, 50, 0, 100, Component.literal("Bounded"),
            Slider.Format.FLOAT, (control, value) -> Math.max(.2, value));
        corrected.setValue(0);
        equal(corrected.getValue(), .2, "Callback-adjusted value must be retained");
        check(corrected.getMessage().getString().contains("20.000"), "Callback correction must precede label update");
        Slider integer = new Slider(0, 0, 108, 20, 10, 0, 100, Component.literal("Integer"), Slider.Format.INT, null);
        integer.setValue(.8);
        equal(integer.scaleValue(integer.getValue()), 80, "Integer formatting retained");
        check(integer.getMessage().getString().contains("80"), "Null callback still updates label");
        Slider continent = new Slider(0, 0, 108, 20, 3000, 100, 10000,
            Component.literal("Continent Scale"), Slider.Format.INT, null);
        check(continent.getMessage().getString().contains("3000"),
            "Initial integer label must retain preset value without float narrowing");
        Bootstrap.realStdoutPrintln("SLIDER_QA_JSON=" + new Gson().toJson(Map.of(
            "status", "pass", "assertions", assertions, "minecraft", SharedConstants.getCurrentVersion().id(),
            "scenarios", List.of("actual_mouse_onClick_callback_and_label", "actual_mouseDragged_callback_and_label",
                "actual_keyboard_right_left_callback_and_label", "normalized_value_clamping", "unchanged_no_extra_callback",
                "callback_correction_order", "integer_format_and_null_callback"),
            "scope", "Real production Slider and actual 26.2 input/superclass methods; no rendered client/window or complete preset editor")));
        Bootstrap.shutdownStdout();
    }
}
