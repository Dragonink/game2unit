# Provides the [Minecraft] Prism Launcher integration.

# Retrive the arguments needed to call `app2unit`.
#
# Returns `null` if the launcher environment is not Prism Launcher.
export def main []: nothing -> oneof<record<app_name: string, description: string>, nothing> {
	let INSTANCE_ID: string = (
		$env.INST_ID?
		| if $in != null { $in } else { return null }
	)
	let INSTANCE_NAME: string = (
		$env.INST_NAME?
		| if $in != null { $in } else { return null }
	)

	{
		app_name: $"minecraft_prism_($INSTANCE_ID)",
		description: $"Minecraft \(($INSTANCE_NAME)\)",
	}
}
