#include <gtest/gtest.h>

#include "hoffman/flake/url-name.hh"
#include "hoffman/util/url.hh"

namespace hoffman {

/* ----------- tests for url-name.hh --------------------------------------------------*/

TEST(getNameFromURL, getNameFromURL)
{
    ASSERT_EQ(getNameFromURL(parseURL("path:/home/user/project")), "project");
    ASSERT_EQ(getNameFromURL(parseURL("path:~/repos/hoffmanpkgs#packages.x86_64-linux.hello")), "hello");
    ASSERT_EQ(getNameFromURL(parseURL("path:~/repos/hoffmanpkgs#legacyPackages.x86_64-linux.hello")), "hello");
    ASSERT_EQ(getNameFromURL(parseURL("path:~/repos/hoffmanpkgs#packages.x86_64-linux.Hello")), "Hello");
    ASSERT_EQ(getNameFromURL(parseURL("path:.#nonStandardAttr.mylaptop")), "mylaptop");
    ASSERT_EQ(getNameFromURL(parseURL("path:./repos/myflake#nonStandardAttr.mylaptop")), "mylaptop");
    ASSERT_EQ(
        getNameFromURL(parseURL("path:./hoffmanpkgs#packages.x86_64-linux.complex^bin,man", /*lenient=*/true)), "complex");
    ASSERT_EQ(getNameFromURL(parseURL("path:./myproj#packages.x86_64-linux.default^*", /*lenient=*/true)), "myproj");
    ASSERT_EQ(getNameFromURL(parseURL("path:./myproj#defaultPackage.x86_64-linux")), "myproj");

    ASSERT_EQ(getNameFromURL(parseURL("github:HoffmanOS/hoffmanpkgs#packages.x86_64-linux.hello")), "hello");
    ASSERT_EQ(getNameFromURL(parseURL("github:HoffmanOS/hoffmanpkgs#hello")), "hello");
    ASSERT_EQ(getNameFromURL(parseURL("github:HoffmanOS/hoffman#packages.x86_64-linux.default")), "hoffman");
    ASSERT_EQ(getNameFromURL(parseURL("github:HoffmanOS/hoffman#")), "hoffman");
    ASSERT_EQ(getNameFromURL(parseURL("github:HoffmanOS/hoffman")), "hoffman");
    ASSERT_EQ(getNameFromURL(parseURL("github:cachix/devenv/main#packages.x86_64-linux.default")), "devenv");
    ASSERT_EQ(getNameFromURL(parseURL("github:edolstra/hoffman-warez?rev=1234&dir=blender&ref=master")), "blender");

    ASSERT_EQ(getNameFromURL(parseURL("gitlab:HoffmanOS/hoffmanpkgs#packages.x86_64-linux.hello")), "hello");
    ASSERT_EQ(getNameFromURL(parseURL("gitlab:HoffmanOS/hoffmanpkgs#hello")), "hello");
    ASSERT_EQ(getNameFromURL(parseURL("gitlab:HoffmanOS/hoffman#packages.x86_64-linux.default")), "hoffman");
    ASSERT_EQ(getNameFromURL(parseURL("gitlab:HoffmanOS/hoffman#")), "hoffman");
    ASSERT_EQ(getNameFromURL(parseURL("gitlab:HoffmanOS/hoffman")), "hoffman");
    ASSERT_EQ(getNameFromURL(parseURL("gitlab:cachix/devenv/main#packages.x86_64-linux.default")), "devenv");

    ASSERT_EQ(getNameFromURL(parseURL("sourcehut:HoffmanOS/hoffmanpkgs#packages.x86_64-linux.hello")), "hello");
    ASSERT_EQ(getNameFromURL(parseURL("sourcehut:HoffmanOS/hoffmanpkgs#hello")), "hello");
    ASSERT_EQ(getNameFromURL(parseURL("sourcehut:HoffmanOS/hoffman#packages.x86_64-linux.default")), "hoffman");
    ASSERT_EQ(getNameFromURL(parseURL("sourcehut:HoffmanOS/hoffman#")), "hoffman");
    ASSERT_EQ(getNameFromURL(parseURL("sourcehut:HoffmanOS/hoffman")), "hoffman");
    ASSERT_EQ(getNameFromURL(parseURL("sourcehut:cachix/devenv/main#packages.x86_64-linux.default")), "devenv");

    ASSERT_EQ(getNameFromURL(parseURL("git://github.com/edolstra/dwarffs")), "dwarffs");
    ASSERT_EQ(getNameFromURL(parseURL("git://github.com/edolstra/hoffman-warez?dir=blender")), "blender");
    ASSERT_EQ(getNameFromURL(parseURL("git+file:///home/user/project")), "project");
    ASSERT_EQ(getNameFromURL(parseURL("git+file:///home/user/project?ref=fa1e2d23a22")), "project");
    ASSERT_EQ(getNameFromURL(parseURL("git+ssh://git@github.com/someuser/my-repo#")), "my-repo");
    ASSERT_EQ(getNameFromURL(parseURL("git+git://github.com/someuser/my-repo?rev=v1.2.3")), "my-repo");
    ASSERT_EQ(getNameFromURL(parseURL("git+ssh:///home/user/project?dir=subproject&rev=v2.4")), "subproject");
    ASSERT_EQ(getNameFromURL(parseURL("git+http://not-even-real#packages.x86_64-linux.hello")), "hello");
    ASSERT_EQ(getNameFromURL(parseURL("git+https://not-even-real#packages.aarch64-darwin.hello")), "hello");

    ASSERT_EQ(
        getNameFromURL(
            parseURL("tarball+http://github.com/HoffmanOS/hoffman/archive/refs/tags/2.18.1#packages.x86_64-linux.jq")),
        "jq");
    ASSERT_EQ(
        getNameFromURL(
            parseURL("tarball+https://github.com/HoffmanOS/hoffman/archive/refs/tags/2.18.1#packages.x86_64-linux.hg")),
        "hg");
    ASSERT_EQ(
        getNameFromURL(parseURL("tarball+file:///home/user/Downloads/hoffmanpkgs-2.18.1#packages.aarch64-darwin.ripgrep")),
        "ripgrep");

    ASSERT_EQ(
        getNameFromURL(
            parseURL("https://github.com/HoffmanOS/hoffman/archive/refs/tags/2.18.1.tar.gz#packages.x86_64-linux.pv")),
        "pv");
    ASSERT_EQ(
        getNameFromURL(
            parseURL("http://github.com/HoffmanOS/hoffman/archive/refs/tags/2.18.1.tar.gz#packages.x86_64-linux.pv")),
        "pv");

    ASSERT_EQ(getNameFromURL(parseURL("file:///home/user/project?ref=fa1e2d23a22")), "project");
    ASSERT_EQ(getNameFromURL(parseURL("file+file:///home/user/project?ref=fa1e2d23a22")), "project");
    ASSERT_EQ(getNameFromURL(parseURL("file+http://not-even-real#packages.x86_64-linux.hello")), "hello");
    ASSERT_EQ(getNameFromURL(parseURL("file+http://gitfantasy.com/org/user/notaflake")), "notaflake");
    ASSERT_EQ(getNameFromURL(parseURL("file+https://not-even-real#packages.aarch64-darwin.hello")), "hello");

    ASSERT_EQ(getNameFromURL(parseURL("https://www.github.com/")), std::nullopt);
    ASSERT_EQ(getNameFromURL(parseURL("path:.")), std::nullopt);
    ASSERT_EQ(getNameFromURL(parseURL("file:.#")), std::nullopt);
    ASSERT_EQ(getNameFromURL(parseURL("path:.#packages.x86_64-linux.default")), std::nullopt);
    ASSERT_EQ(getNameFromURL(parseURL("path:.#packages.x86_64-linux.default^*", /*lenient=*/true)), std::nullopt);
}
} // namespace hoffman
