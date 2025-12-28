package com.kosac.ai.codegen.preferences;

import org.eclipse.jface.preference.BooleanFieldEditor;
import org.eclipse.jface.preference.ComboFieldEditor;
import org.eclipse.jface.preference.FieldEditorPreferencePage;
import org.eclipse.jface.preference.IntegerFieldEditor;
import org.eclipse.jface.preference.StringFieldEditor;
import org.eclipse.ui.IWorkbench;
import org.eclipse.ui.IWorkbenchPreferencePage;

import com.kosac.ai.codegen.Activator;

/**
 * Preference page for Enterprise Code Generator.
 *
 * Note: This page intentionally does NOT include any LLM configuration options
 * (model selection, temperature, etc.) because:
 * 1. The plugin is "dumb" and doesn't know about LLM details
 * 2. LLM configuration is managed server-side
 * 3. This provides proper abstraction per CLAUDE.md requirements
 */
public class MainPreferencePage extends FieldEditorPreferencePage implements IWorkbenchPreferencePage {

    public MainPreferencePage() {
        super(GRID);
        setPreferenceStore(Activator.getDefault().getPreferenceStore());
        setDescription("Configure the Code Generator plugin.");
    }

    @Override
    public void createFieldEditors() {
        // Server settings
        addField(new StringFieldEditor(
                PreferenceConstants.P_SERVER_ENDPOINT,
                "Agent Server URL:",
                getFieldEditorParent()));

        addField(new IntegerFieldEditor(
                PreferenceConstants.P_TIMEOUT,
                "Timeout (seconds):",
                getFieldEditorParent()));

        // Company settings
        addField(new StringFieldEditor(
                PreferenceConstants.P_COMPANY_ID,
                "Company ID (optional):",
                getFieldEditorParent()));

        // Generation settings
        addField(new ComboFieldEditor(
                PreferenceConstants.P_LANGUAGE,
                "Output Language:",
                new String[][] {
                        {"Korean", "ko"},
                        {"English", "en"}
                },
                getFieldEditorParent()));

        addField(new BooleanFieldEditor(
                PreferenceConstants.P_STRICT_MODE,
                "Strict Validation Mode",
                getFieldEditorParent()));

        // Output folders
        addField(new StringFieldEditor(
                PreferenceConstants.P_VIEWS_FOLDER,
                "Views Folder:",
                getFieldEditorParent()));

        addField(new StringFieldEditor(
                PreferenceConstants.P_SCRIPTS_FOLDER,
                "Scripts Folder:",
                getFieldEditorParent()));
    }

    @Override
    public void init(IWorkbench workbench) {
        // Nothing to initialize
    }
}
