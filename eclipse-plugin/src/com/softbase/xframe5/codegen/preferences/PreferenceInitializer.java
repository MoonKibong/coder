package com.softbase.xframe5.codegen.preferences;

import org.eclipse.core.runtime.preferences.AbstractPreferenceInitializer;
import org.eclipse.jface.preference.IPreferenceStore;

import com.softbase.xframe5.codegen.Activator;

/**
 * Initializes default preference values.
 */
public class PreferenceInitializer extends AbstractPreferenceInitializer {

    @Override
    public void initializeDefaultPreferences() {
        IPreferenceStore store = Activator.getDefault().getPreferenceStore();

        store.setDefault(PreferenceConstants.P_SERVER_ENDPOINT,
                PreferenceConstants.DEFAULT_SERVER_ENDPOINT);

        store.setDefault(PreferenceConstants.P_TIMEOUT,
                PreferenceConstants.DEFAULT_TIMEOUT);

        store.setDefault(PreferenceConstants.P_LANGUAGE,
                PreferenceConstants.DEFAULT_LANGUAGE);

        store.setDefault(PreferenceConstants.P_STRICT_MODE, false);

        store.setDefault(PreferenceConstants.P_VIEWS_FOLDER,
                PreferenceConstants.DEFAULT_VIEWS_FOLDER);

        store.setDefault(PreferenceConstants.P_SCRIPTS_FOLDER,
                PreferenceConstants.DEFAULT_SCRIPTS_FOLDER);
    }
}
