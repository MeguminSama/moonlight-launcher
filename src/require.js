const { app } = require("electron");

// This redirects the relaunch function to launch the modloader executable instead of the original executable
// There is a bug on Linux where Discord does not relaunch. This is an electron bug, and I haven't figured out how to fix it yet.
// See: https://github.com/electron/electron/issues/41463
if (process.env.MODLOADER_EXECUTABLE) {
  const _relaunch = app.relaunch;

  app.relaunch = (options = {}) => {
    let args = process.argv.slice(1);
    try {
      args = JSON.parse(process.env.MODLOADER_PROCESS_ARGV);
    } catch (e) {
      console.error("Failed to parse MODLOADER_PROCESS_ARGV", e);
    }

    _relaunch.call(app, {
      args,
      execPath: process.env.MODLOADER_EXECUTABLE,
    });
  };
}

// discord will try to set the name to `discord-{branch}` so we intercept that and change it to `moonlight-{branch}` so that the wm_class is correct
{
  const _setName = app.setName;

  app.setName = (name) => {
    name = name.replace("discord", "moonlight").replace("Discord", "Moonlight");
    _setName.call(app, name);
  };

  const _setDesktopName = app.setDesktopName;

  app.setDesktopName = (name) => {
    name = name.replace("discord", "moonlight").replace("Discord", "Moonlight");
    _setDesktopName.call(app, name);
  };
}

require(process.env.MODLOADER_MOD_ENTRYPOINT).inject(
  require("path").resolve(
    __dirname,
    process.env.MODLOADER_ORIGINAL_ASAR_RELATIVE,
  ), {
  disablePersist: true
}
);
