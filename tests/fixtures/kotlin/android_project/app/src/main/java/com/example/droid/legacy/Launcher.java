package com.example.droid.legacy;

import android.content.Context;
import com.example.droid.widgets.BadgeView;

public class Launcher {
    public BadgeView create(Context context) {
        BadgeView badge = new BadgeView(context, null);
        badge.setCount(7);
        return badge;
    }
}
