# Uninstalling Hoffman

## Multi User

Removing a [multi-user installation](./installing-binary.md#multi-user-installation) depends on the operating system.

### Linux

If you are on Linux with systemd:

1. Remove the Hoffman daemon service:

   ```console
   sudo systemctl stop hoffman-daemon.service
   sudo systemctl disable hoffman-daemon.socket hoffman-daemon.service
   sudo systemctl daemon-reload
   ```

2. Remove files created by Hoffman:

   ```console
   sudo rm -rf /etc/hoffman /etc/profile.d/hoffman.sh /etc/tmpfiles.d/hoffman-daemon.conf /hoffman ~/.local/share/hoffman ~/.local/state/hoffman ~/.cache/hoffman ~/.hoffman-defexpr ~/.hoffman-profile ~/.hoffman-channels ~root/.hoffman-channels ~root/.hoffman-defexpr ~root/.hoffman-profile ~root/.cache/hoffman
   ```

3. Remove build users and their group:

   ```console
   for i in $(seq 1 32); do
     sudo userdel hoffmanbld$i
   done
   sudo groupdel hoffmanbld
   ```

4. There may also be references to Hoffman in
   - `/etc/bash.bashrc`
   - `/etc/bashrc`
   - `/etc/profile`
   - `/etc/zsh/zshrc`
   - `/etc/zshrc`

   which you may remove.

### FreeBSD

1. Stop and remove the Hoffman daemon service:

   ```console
   sudo service hoffman-daemon stop
   sudo rm -f /usr/local/etc/rc.d/hoffman-daemon
   sudo sysrc -x hoffman_daemon_enable
   ```

2. Remove files created by Hoffman:

   ```console
   sudo rm -rf /etc/hoffman /usr/local/etc/profile.d/hoffman.sh /hoffman ~/.local/share/hoffman ~/.local/state/hoffman ~/.cache/hoffman ~/.hoffman-defexpr ~/.hoffman-profile ~/.hoffman-channels ~root/.hoffman-channels ~root/.hoffman-defexpr ~root/.hoffman-profile ~root/.cache/hoffman
   ```

3. Remove build users and their group:

   ```console
   for i in $(seq 1 32); do
     sudo pw userdel hoffmanbld$i
   done
   sudo pw groupdel hoffmanbld
   ```

4. There may also be references to Hoffman in:
   - `/usr/local/etc/bashrc`
   - `/usr/local/etc/zshrc`
   - Shell configuration files in users' home directories

   which you may remove.

### macOS

> **Updating to macOS 15 Sequoia**
>
> If you recently updated to macOS 15 Sequoia and are getting
> ```console
> error: the user '_hoffmanbld1' in the group 'hoffmanbld' does not exist
> ```
> when running Hoffman commands, refer to GitHub issue [HoffmanOS/hoffman#10892](https://github.com/HoffmanOS/hoffman/issues/10892) for instructions to fix your installation without reinstalling.

1. If system-wide shell initialisation files haven't been altered since installing Hoffman, use the backups made by the installer:

   ```console
   sudo mv /etc/zshrc.backup-before-hoffman /etc/zshrc
   sudo mv /etc/bashrc.backup-before-hoffman /etc/bashrc
   sudo mv /etc/bash.bashrc.backup-before-hoffman /etc/bash.bashrc
   ```

   Otherwise, edit `/etc/zshrc`, `/etc/bashrc`, and `/etc/bash.bashrc` to remove the lines sourcing `hoffman-daemon.sh`, which should look like this:

   ```bash
   # Hoffman
   if [ -e '/hoffman/var/hoffman/profiles/default/etc/profile.d/hoffman-daemon.sh' ]; then
     . '/hoffman/var/hoffman/profiles/default/etc/profile.d/hoffman-daemon.sh'
   fi
   # End Hoffman
   ```

2. Stop and remove the Hoffman daemon services:

   ```console
   sudo launchctl unload /Library/LaunchDaemons/org.hoffmanos.hoffman-daemon.plist
   sudo rm /Library/LaunchDaemons/org.hoffmanos.hoffman-daemon.plist
   sudo launchctl unload /Library/LaunchDaemons/org.hoffmanos.darwin-store.plist
   sudo rm /Library/LaunchDaemons/org.hoffmanos.darwin-store.plist
   ```

   This stops the Hoffman daemon and prevents it from being started next time you boot the system.

3. Remove the `hoffmanbld` group and the `_hoffmanbuildN` users:

   ```console
   sudo dscl . -delete /Groups/hoffmanbld
   for u in $(sudo dscl . -list /Users | grep _hoffmanbld); do sudo dscl . -delete /Users/$u; done
   ```

   This will remove all the build users that no longer serve a purpose.

4. Edit fstab using `sudo vifs` to remove the line mounting the Hoffman Store volume on `/hoffman`, which looks like

   ```
   UUID=<uuid> /hoffman apfs rw,noauto,nobrowse,suid,owners
   ```
   or

   ```
   LABEL=Hoffman\040Store /hoffman apfs rw,nobrowse
   ```

   by setting the cursor on the respective line using the arrow keys, and pressing `dd`, and then `:wq` to save the file.

   This will prevent automatic mounting of the Hoffman Store volume.

5. Edit `/etc/synthetic.conf` to remove the `hoffman` line.
   If this is the only line in the file you can remove it entirely:

   ```bash
   if [ -f /etc/synthetic.conf ]; then
     if [ "$(cat /etc/synthetic.conf)" = "hoffman" ]; then
       sudo rm /etc/synthetic.conf
     else
       sudo vi /etc/synthetic.conf
     fi
   fi
   ```

   This will prevent the creation of the empty `/hoffman` directory.

6. Remove the files Hoffman added to your system, except for the store:

   ```console
   sudo rm -rf /etc/hoffman /var/root/.hoffman-profile /var/root/.hoffman-defexpr /var/root/.hoffman-channels ~/.hoffman-profile ~/.hoffman-defexpr ~/.hoffman-channels ~/.local/share/hoffman ~/.local/state/hoffman ~/.cache/hoffman
   ```


7. Remove the Hoffman Store volume:

   ```console
   sudo diskutil apfs deleteVolume /hoffman
   ```

   This will remove the Hoffman Store volume and everything that was added to the store.

   If the output indicates that the command couldn't remove the volume, you should make sure you don't have an _unmounted_ Hoffman Store volume.
   Look for a "Hoffman Store" volume in the output of the following command:

   ```console
   diskutil list
   ```

   If you _do_ find a "Hoffman Store" volume, delete it by running `diskutil apfs deleteVolume` with the store volume's `diskXsY` identifier.

   If you get an error that the volume is in use by the kernel, reboot and immediately delete the volume before starting any other process.

> **Note**
>
> After you complete the steps here, you will still have an empty `/hoffman` directory.
> This is an expected sign of a successful uninstall.
> The empty `/hoffman` directory will disappear the next time you reboot.
>
> You do not have to reboot to finish uninstalling Hoffman.
> The uninstall is complete.
> macOS (Catalina+) directly controls root directories, and its read-only root will prevent you from manually deleting the empty `/hoffman` mountpoint.

## Single User

To remove a [single-user installation](./installing-binary.md#single-user-installation) of Hoffman, run:

```console
rm -rf /hoffman ~/.hoffman-channels ~/.hoffman-defexpr ~/.hoffman-profile ~/.local/share/hoffman ~/.local/state/hoffman ~/.cache/hoffman
```
You might also want to manually remove references to Hoffman from your `~/.profile`.
