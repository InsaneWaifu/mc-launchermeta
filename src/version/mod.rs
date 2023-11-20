
////////////////////////////////////////////////////////////////////////////////
// Copyright (c) 2023. Rob Bailey                                              /
// This Source Code Form is subject to the terms of the Mozilla Public         /
// License, v. 2.0. If a copy of the MPL was not distributed with this         /
// file, You can obtain one at https://mozilla.org/MPL/2.0/.                   /
////////////////////////////////////////////////////////////////////////////////

mod rule;
mod logging;
mod library;

use std::fmt;
use std::str::FromStr;
use serde::{de, Deserialize, Deserializer, Serialize};
use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use library::Library;
use logging::Logging;
use rule::Rule;
use crate::VersionKind;


#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize)]
pub struct Argument {
    pub rules: Vec<Rule>,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct ArrayOrStringHelper(pub Vec<String>);

/// deserialize either an array of strings or a single string into always a vector of strings
impl<'de> Deserialize<'de> for ArrayOrStringHelper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        struct ArrayOrStringVisitor;

        impl<'de> Visitor<'de> for ArrayOrStringVisitor {
            type Value = ArrayOrStringHelper;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string or array of strings")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
                where E: de::Error
            {
                Ok(ArrayOrStringHelper(vec![s.to_owned()]))
            }

            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
                where S: SeqAccess<'de>
            {
                let mut vec = Vec::new();
                while let Some(elem) = seq.next_element::<String>()? {
                    vec.push(elem);
                }
                Ok(ArrayOrStringHelper(vec))
            }
        }

        deserializer.deserialize_any(ArrayOrStringVisitor)
    }
}

impl FromStr for Argument {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Argument {
            rules: vec![],
            values: vec![s.to_owned()],
        })
    }
}

impl<'de> Deserialize<'de> for Argument {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        struct ArgumentVisitor;

        impl<'de> Visitor<'de> for ArgumentVisitor {
            type Value = Argument;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string or object with rules and value fields")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
                where E: de::Error
            {
                Ok(Argument {
                    rules: vec![],
                    values: vec![s.to_owned()],
                })
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
                where M: MapAccess<'de>
            {
                let mut rules = None;
                let mut value = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "rules" => {
                            if rules.is_some() {
                                return Err(de::Error::duplicate_field("rules"));
                            }
                            rules = Some(map.next_value()?);
                        }
                        "value" => {
                            if value.is_some() {
                                return Err(de::Error::duplicate_field("value"));
                            }
                            value = Some(map.next_value::<ArrayOrStringHelper>()?.0);
                        }
                        _ => {
                            return Err(Error::unknown_field(&key, &["rules", "value"]));
                        }
                    }
                }

                let rules = rules.ok_or_else(|| de::Error::missing_field("rules"))?;
                let value = value.ok_or_else(|| de::Error::missing_field("value"))?;

                Ok(Argument {
                    rules,
                    values: value,
                })
            }
        }

        deserializer.deserialize_any(ArgumentVisitor)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Arguments {
    pub game: Vec<Argument>,
    pub jvm: Vec<Argument>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub total_size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Download {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Downloads {
    pub client: Download,
    #[serde(default)]
    pub client_mappings: Option<Download>,
    #[serde(default)]
    pub server: Option<Download>,
    #[serde(default)]
    pub server_mappings: Option<Download>,
    #[serde(default)]
    pub windows_server: Option<Download>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct JavaVersion {
    pub component: String,
    pub major_version: u8,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Version {
    #[serde(default)]
    pub arguments: Option<Arguments>,
    #[serde(default)]
    pub minecraft_arguments: Option<String>,
    pub asset_index: AssetIndex,
    pub assets: String,
    #[serde(default)]
    pub compliance_level: Option<u8>,
    pub downloads: Downloads,
    pub id: String,
    #[serde(default)]
    pub java_version: Option<JavaVersion>,
    pub libraries: Vec<Library>,
    #[serde(default)]
    pub logging: Option<Logging>,
    pub main_class: String,
    pub minimum_launcher_version: u8,
    pub release_time: String,
    pub time: String,
    #[serde(rename = "type")]
    pub kind: VersionKind,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn single_argument_test() {
        // simple string
        let raw = r#""--username""#;
        let arg: Argument = serde_json::from_str(raw).unwrap();
        println!("{:?}", arg);
        // simple object
        let raw = r#"{"rules": [], "value": "--username"}"#;
        let arg: Argument = serde_json::from_str(raw).unwrap();
        println!("{:?}", arg);
        // object with defined rules and single value
        let raw = r#"{"rules": [{"action": "allow", "features": {"is_demo_user": true}}], "value": "--demo"}"#;
        let arg: Argument = serde_json::from_str(raw).unwrap();
        println!("{:?}", arg);
        // object with defined rules and multiple values
        let raw = r#"{"rules": [{"action": "allow", "features": {"has_custom_resolution": true}}], "value": ["--width", "${resolution_width}", "--height", "${resolution_height}"]}"#;
        let arg: Argument = serde_json::from_str(raw).unwrap();
        println!("{:?}", arg);
    }

    #[test]
    fn arguments_test() {
        let raw = r#"{
    "game": [
      "--username",
      "${auth_player_name}",
      "--version",
      "${version_name}",
      "--gameDir",
      "${game_directory}",
      "--assetsDir",
      "${assets_root}",
      "--assetIndex",
      "${assets_index_name}",
      "--uuid",
      "${auth_uuid}",
      "--accessToken",
      "${auth_access_token}",
      "--clientId",
      "${clientid}",
      "--xuid",
      "${auth_xuid}",
      "--userType",
      "${user_type}",
      "--versionType",
      "${version_type}",
      {
        "rules": [
          {
            "action": "allow",
            "features": {
              "is_demo_user": true
            }
          }
        ],
        "value": "--demo"
      },
      {
        "rules": [
          {
            "action": "allow",
            "features": {
              "has_custom_resolution": true
            }
          }
        ],
        "value": [
          "--width",
          "${resolution_width}",
          "--height",
          "${resolution_height}"
        ]
      },
      {
        "rules": [
          {
            "action": "allow",
            "features": {
              "has_quick_plays_support": true
            }
          }
        ],
        "value": [
          "--quickPlayPath",
          "${quickPlayPath}"
        ]
      },
      {
        "rules": [
          {
            "action": "allow",
            "features": {
              "is_quick_play_singleplayer": true
            }
          }
        ],
        "value": [
          "--quickPlaySingleplayer",
          "${quickPlaySingleplayer}"
        ]
      },
      {
        "rules": [
          {
            "action": "allow",
            "features": {
              "is_quick_play_multiplayer": true
            }
          }
        ],
        "value": [
          "--quickPlayMultiplayer",
          "${quickPlayMultiplayer}"
        ]
      },
      {
        "rules": [
          {
            "action": "allow",
            "features": {
              "is_quick_play_realms": true
            }
          }
        ],
        "value": [
          "--quickPlayRealms",
          "${quickPlayRealms}"
        ]
      }
    ],
    "jvm": [
      {
        "rules": [
          {
            "action": "allow",
            "os": {
              "name": "osx"
            }
          }
        ],
        "value": [
          "-XstartOnFirstThread"
        ]
      },
      {
        "rules": [
          {
            "action": "allow",
            "os": {
              "name": "windows"
            }
          }
        ],
        "value": "-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump"
      },
      {
        "rules": [
          {
            "action": "allow",
            "os": {
              "arch": "x86"
            }
          }
        ],
        "value": "-Xss1M"
      },
      "-Djava.library.path=${natives_directory}",
      "-Djna.tmpdir=${natives_directory}",
      "-Dorg.lwjgl.system.SharedLibraryExtractPath=${natives_directory}",
      "-Dio.netty.native.workdir=${natives_directory}",
      "-Dminecraft.launcher.brand=${launcher_name}",
      "-Dminecraft.launcher.version=${launcher_version}",
      "-cp",
      "${classpath}"
    ]
  }"#;

        let args: Arguments = serde_json::from_str(raw).unwrap();
        println!("{:?}", args);

    }

    #[test]
    fn asset_index_test() {
        let raw = r#"{
    "id": "11",
    "sha1": "1e62f8db74422c8ceec551b5cbf98414d34c24b3",
    "size": 426900,
    "totalSize": 623629518,
    "url": "https://piston-meta.mojang.com/v1/packages/1e62f8db74422c8ceec551b5cbf98414d34c24b3/11.json"
    }"#;

            let index: AssetIndex = serde_json::from_str(raw).unwrap();
            println!("{:?}", index);
        }

    #[test]
    fn final_test() {
        let raw = r#"{"arguments": {"game": ["--username", "${auth_player_name}", "--version", "${version_name}", "--gameDir", "${game_directory}", "--assetsDir", "${assets_root}", "--assetIndex", "${assets_index_name}", "--uuid", "${auth_uuid}", "--accessToken", "${auth_access_token}", "--clientId", "${clientid}", "--xuid", "${auth_xuid}", "--userType", "${user_type}", "--versionType", "${version_type}", {"rules": [{"action": "allow", "features": {"is_demo_user": true}}], "value": "--demo"}, {"rules": [{"action": "allow", "features": {"has_custom_resolution": true}}], "value": ["--width", "${resolution_width}", "--height", "${resolution_height}"]}, {"rules": [{"action": "allow", "features": {"has_quick_plays_support": true}}], "value": ["--quickPlayPath", "${quickPlayPath}"]}, {"rules": [{"action": "allow", "features": {"is_quick_play_singleplayer": true}}], "value": ["--quickPlaySingleplayer", "${quickPlaySingleplayer}"]}, {"rules": [{"action": "allow", "features": {"is_quick_play_multiplayer": true}}], "value": ["--quickPlayMultiplayer", "${quickPlayMultiplayer}"]}, {"rules": [{"action": "allow", "features": {"is_quick_play_realms": true}}], "value": ["--quickPlayRealms", "${quickPlayRealms}"]}], "jvm": [{"rules": [{"action": "allow", "os": {"name": "osx"}}], "value": ["-XstartOnFirstThread"]}, {"rules": [{"action": "allow", "os": {"name": "windows"}}], "value": "-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump"}, {"rules": [{"action": "allow", "os": {"arch": "x86"}}], "value": "-Xss1M"}, "-Djava.library.path=${natives_directory}", "-Djna.tmpdir=${natives_directory}", "-Dorg.lwjgl.system.SharedLibraryExtractPath=${natives_directory}", "-Dio.netty.native.workdir=${natives_directory}", "-Dminecraft.launcher.brand=${launcher_name}", "-Dminecraft.launcher.version=${launcher_version}", "-cp", "${classpath}"]}, "assetIndex": {"id": "11", "sha1": "1e62f8db74422c8ceec551b5cbf98414d34c24b3", "size": 426900, "totalSize": 623629518, "url": "https://piston-meta.mojang.com/v1/packages/1e62f8db74422c8ceec551b5cbf98414d34c24b3/11.json"}, "assets": "11", "complianceLevel": 1, "downloads": {"client": {"sha1": "265ca2072f7c3a9e0dae8c4abe223431089d9980", "size": 24339738, "url": "https://piston-data.mojang.com/v1/objects/265ca2072f7c3a9e0dae8c4abe223431089d9980/client.jar"}, "client_mappings": {"sha1": "15bd31430e6903a34c68950d9443026f991a143e", "size": 8835386, "url": "https://piston-data.mojang.com/v1/objects/15bd31430e6903a34c68950d9443026f991a143e/client.txt"}, "server": {"sha1": "9c2b37701bf77ae22df4c32fd6dd1614049ce994", "size": 49093592, "url": "https://piston-data.mojang.com/v1/objects/9c2b37701bf77ae22df4c32fd6dd1614049ce994/server.jar"}, "server_mappings": {"sha1": "56b78613aeed0b0c38b887e1ac7e948dc5dbc236", "size": 6759381, "url": "https://piston-data.mojang.com/v1/objects/56b78613aeed0b0c38b887e1ac7e948dc5dbc236/server.txt"}}, "id": "23w45a", "javaVersion": {"component": "java-runtime-gamma", "majorVersion": 17}, "libraries": [{"downloads": {"artifact": {"path": "ca/weblite/java-objc-bridge/1.1/java-objc-bridge-1.1.jar", "sha1": "1227f9e0666314f9de41477e3ec277e542ed7f7b", "size": 1330045, "url": "https://libraries.minecraft.net/ca/weblite/java-objc-bridge/1.1/java-objc-bridge-1.1.jar"}}, "name": "ca.weblite:java-objc-bridge:1.1", "rules": [{"action": "allow", "os": {"name": "osx"}}]}, {"downloads": {"artifact": {"path": "com/github/oshi/oshi-core/6.4.5/oshi-core-6.4.5.jar", "sha1": "943ba26de047eb6b28fff47f5ee939a34eb5fc8e", "size": 970546, "url": "https://libraries.minecraft.net/com/github/oshi/oshi-core/6.4.5/oshi-core-6.4.5.jar"}}, "name": "com.github.oshi:oshi-core:6.4.5"}, {"downloads": {"artifact": {"path": "com/google/code/gson/gson/2.10.1/gson-2.10.1.jar", "sha1": "b3add478d4382b78ea20b1671390a858002feb6c", "size": 283367, "url": "https://libraries.minecraft.net/com/google/code/gson/gson/2.10.1/gson-2.10.1.jar"}}, "name": "com.google.code.gson:gson:2.10.1"}, {"downloads": {"artifact": {"path": "com/google/guava/failureaccess/1.0.1/failureaccess-1.0.1.jar", "sha1": "1dcf1de382a0bf95a3d8b0849546c88bac1292c9", "size": 4617, "url": "https://libraries.minecraft.net/com/google/guava/failureaccess/1.0.1/failureaccess-1.0.1.jar"}}, "name": "com.google.guava:failureaccess:1.0.1"}, {"downloads": {"artifact": {"path": "com/google/guava/guava/32.1.2-jre/guava-32.1.2-jre.jar", "sha1": "5e64ec7e056456bef3a4bc4c6fdaef71e8ab6318", "size": 3041591, "url": "https://libraries.minecraft.net/com/google/guava/guava/32.1.2-jre/guava-32.1.2-jre.jar"}}, "name": "com.google.guava:guava:32.1.2-jre"}, {"downloads": {"artifact": {"path": "com/ibm/icu/icu4j/73.2/icu4j-73.2.jar", "sha1": "61ad4ef7f9131fcf6d25c34b817f90d6da06c9e9", "size": 14567819, "url": "https://libraries.minecraft.net/com/ibm/icu/icu4j/73.2/icu4j-73.2.jar"}}, "name": "com.ibm.icu:icu4j:73.2"}, {"downloads": {"artifact": {"path": "com/mojang/authlib/5.0.51/authlib-5.0.51.jar", "sha1": "83c235be98157238ce6ebec8d6454ef94507360e", "size": 114272, "url": "https://libraries.minecraft.net/com/mojang/authlib/5.0.51/authlib-5.0.51.jar"}}, "name": "com.mojang:authlib:5.0.51"}, {"downloads": {"artifact": {"path": "com/mojang/blocklist/1.0.10/blocklist-1.0.10.jar", "sha1": "5c685c5ffa94c4cd39496c7184c1d122e515ecef", "size": 964, "url": "https://libraries.minecraft.net/com/mojang/blocklist/1.0.10/blocklist-1.0.10.jar"}}, "name": "com.mojang:blocklist:1.0.10"}, {"downloads": {"artifact": {"path": "com/mojang/brigadier/1.2.9/brigadier-1.2.9.jar", "sha1": "73e324f2ee541493a5179abf367237faa782ed21", "size": 79955, "url": "https://libraries.minecraft.net/com/mojang/brigadier/1.2.9/brigadier-1.2.9.jar"}}, "name": "com.mojang:brigadier:1.2.9"}, {"downloads": {"artifact": {"path": "com/mojang/datafixerupper/6.0.8/datafixerupper-6.0.8.jar", "sha1": "3ba4a30557a9b057760af4011f909ba619fc5125", "size": 689960, "url": "https://libraries.minecraft.net/com/mojang/datafixerupper/6.0.8/datafixerupper-6.0.8.jar"}}, "name": "com.mojang:datafixerupper:6.0.8"}, {"downloads": {"artifact": {"path": "com/mojang/logging/1.1.1/logging-1.1.1.jar", "sha1": "832b8e6674a9b325a5175a3a6267dfaf34c85139", "size": 15343, "url": "https://libraries.minecraft.net/com/mojang/logging/1.1.1/logging-1.1.1.jar"}}, "name": "com.mojang:logging:1.1.1"}, {"downloads": {"artifact": {"path": "com/mojang/patchy/2.2.10/patchy-2.2.10.jar", "sha1": "da05971b07cbb379d002cf7eaec6a2048211fefc", "size": 4439, "url": "https://libraries.minecraft.net/com/mojang/patchy/2.2.10/patchy-2.2.10.jar"}}, "name": "com.mojang:patchy:2.2.10"}, {"downloads": {"artifact": {"path": "com/mojang/text2speech/1.17.9/text2speech-1.17.9.jar", "sha1": "3cad216e3a7f0c19b4b394388bc9ffc446f13b14", "size": 12243, "url": "https://libraries.minecraft.net/com/mojang/text2speech/1.17.9/text2speech-1.17.9.jar"}}, "name": "com.mojang:text2speech:1.17.9"}, {"downloads": {"artifact": {"path": "commons-codec/commons-codec/1.16.0/commons-codec-1.16.0.jar", "sha1": "4e3eb3d79888d76b54e28b350915b5dc3919c9de", "size": 360738, "url": "https://libraries.minecraft.net/commons-codec/commons-codec/1.16.0/commons-codec-1.16.0.jar"}}, "name": "commons-codec:commons-codec:1.16.0"}, {"downloads": {"artifact": {"path": "commons-io/commons-io/2.13.0/commons-io-2.13.0.jar", "sha1": "8bb2bc9b4df17e2411533a0708a69f983bf5e83b", "size": 483954, "url": "https://libraries.minecraft.net/commons-io/commons-io/2.13.0/commons-io-2.13.0.jar"}}, "name": "commons-io:commons-io:2.13.0"}, {"downloads": {"artifact": {"path": "commons-logging/commons-logging/1.2/commons-logging-1.2.jar", "sha1": "4bfc12adfe4842bf07b657f0369c4cb522955686", "size": 61829, "url": "https://libraries.minecraft.net/commons-logging/commons-logging/1.2/commons-logging-1.2.jar"}}, "name": "commons-logging:commons-logging:1.2"}, {"downloads": {"artifact": {"path": "io/netty/netty-buffer/4.1.97.Final/netty-buffer-4.1.97.Final.jar", "sha1": "f8f3d8644afa5e6e1a40a3a6aeb9d9aa970ecb4f", "size": 306590, "url": "https://libraries.minecraft.net/io/netty/netty-buffer/4.1.97.Final/netty-buffer-4.1.97.Final.jar"}}, "name": "io.netty:netty-buffer:4.1.97.Final"}, {"downloads": {"artifact": {"path": "io/netty/netty-codec/4.1.97.Final/netty-codec-4.1.97.Final.jar", "sha1": "384ba4d75670befbedb45c4d3b497a93639c206d", "size": 345274, "url": "https://libraries.minecraft.net/io/netty/netty-codec/4.1.97.Final/netty-codec-4.1.97.Final.jar"}}, "name": "io.netty:netty-codec:4.1.97.Final"}, {"downloads": {"artifact": {"path": "io/netty/netty-common/4.1.97.Final/netty-common-4.1.97.Final.jar", "sha1": "7cceacaf11df8dc63f23d0fb58e9d4640fc88404", "size": 659930, "url": "https://libraries.minecraft.net/io/netty/netty-common/4.1.97.Final/netty-common-4.1.97.Final.jar"}}, "name": "io.netty:netty-common:4.1.97.Final"}, {"downloads": {"artifact": {"path": "io/netty/netty-handler/4.1.97.Final/netty-handler-4.1.97.Final.jar", "sha1": "abb86c6906bf512bf2b797a41cd7d2e8d3cd7c36", "size": 560040, "url": "https://libraries.minecraft.net/io/netty/netty-handler/4.1.97.Final/netty-handler-4.1.97.Final.jar"}}, "name": "io.netty:netty-handler:4.1.97.Final"}, {"downloads": {"artifact": {"path": "io/netty/netty-resolver/4.1.97.Final/netty-resolver-4.1.97.Final.jar", "sha1": "cec8348108dc76c47cf87c669d514be52c922144", "size": 37792, "url": "https://libraries.minecraft.net/io/netty/netty-resolver/4.1.97.Final/netty-resolver-4.1.97.Final.jar"}}, "name": "io.netty:netty-resolver:4.1.97.Final"}, {"downloads": {"artifact": {"path": "io/netty/netty-transport-classes-epoll/4.1.97.Final/netty-transport-classes-epoll-4.1.97.Final.jar", "sha1": "795da37ded759e862457a82d9d92c4d39ce8ecee", "size": 147139, "url": "https://libraries.minecraft.net/io/netty/netty-transport-classes-epoll/4.1.97.Final/netty-transport-classes-epoll-4.1.97.Final.jar"}}, "name": "io.netty:netty-transport-classes-epoll:4.1.97.Final"}, {"downloads": {"artifact": {"path": "io/netty/netty-transport-native-epoll/4.1.97.Final/netty-transport-native-epoll-4.1.97.Final-linux-aarch_64.jar", "sha1": "5514744c588190ffda076b35a9b8c9f24946a960", "size": 40427, "url": "https://libraries.minecraft.net/io/netty/netty-transport-native-epoll/4.1.97.Final/netty-transport-native-epoll-4.1.97.Final-linux-aarch_64.jar"}}, "name": "io.netty:netty-transport-native-epoll:4.1.97.Final:linux-aarch_64", "rules": [{"action": "allow", "os": {"name": "linux"}}]}, {"downloads": {"artifact": {"path": "io/netty/netty-transport-native-epoll/4.1.97.Final/netty-transport-native-epoll-4.1.97.Final-linux-x86_64.jar", "sha1": "54188f271e388e7f313aea995e82f58ce2cdb809", "size": 38954, "url": "https://libraries.minecraft.net/io/netty/netty-transport-native-epoll/4.1.97.Final/netty-transport-native-epoll-4.1.97.Final-linux-x86_64.jar"}}, "name": "io.netty:netty-transport-native-epoll:4.1.97.Final:linux-x86_64", "rules": [{"action": "allow", "os": {"name": "linux"}}]}, {"downloads": {"artifact": {"path": "io/netty/netty-transport-native-unix-common/4.1.97.Final/netty-transport-native-unix-common-4.1.97.Final.jar", "sha1": "d469d84265ab70095b01b40886cabdd433b6e664", "size": 43897, "url": "https://libraries.minecraft.net/io/netty/netty-transport-native-unix-common/4.1.97.Final/netty-transport-native-unix-common-4.1.97.Final.jar"}}, "name": "io.netty:netty-transport-native-unix-common:4.1.97.Final"}, {"downloads": {"artifact": {"path": "io/netty/netty-transport/4.1.97.Final/netty-transport-4.1.97.Final.jar", "sha1": "f37380d23c9bb079bc702910833b2fd532c9abd0", "size": 489624, "url": "https://libraries.minecraft.net/io/netty/netty-transport/4.1.97.Final/netty-transport-4.1.97.Final.jar"}}, "name": "io.netty:netty-transport:4.1.97.Final"}, {"downloads": {"artifact": {"path": "it/unimi/dsi/fastutil/8.5.12/fastutil-8.5.12.jar", "sha1": "c24946d46824bd528054bface3231d2ecb7e95e8", "size": 23326598, "url": "https://libraries.minecraft.net/it/unimi/dsi/fastutil/8.5.12/fastutil-8.5.12.jar"}}, "name": "it.unimi.dsi:fastutil:8.5.12"}, {"downloads": {"artifact": {"path": "net/java/dev/jna/jna-platform/5.13.0/jna-platform-5.13.0.jar", "sha1": "88e9a306715e9379f3122415ef4ae759a352640d", "size": 1363209, "url": "https://libraries.minecraft.net/net/java/dev/jna/jna-platform/5.13.0/jna-platform-5.13.0.jar"}}, "name": "net.java.dev.jna:jna-platform:5.13.0"}, {"downloads": {"artifact": {"path": "net/java/dev/jna/jna/5.13.0/jna-5.13.0.jar", "sha1": "1200e7ebeedbe0d10062093f32925a912020e747", "size": 1879325, "url": "https://libraries.minecraft.net/net/java/dev/jna/jna/5.13.0/jna-5.13.0.jar"}}, "name": "net.java.dev.jna:jna:5.13.0"}, {"downloads": {"artifact": {"path": "net/sf/jopt-simple/jopt-simple/5.0.4/jopt-simple-5.0.4.jar", "sha1": "4fdac2fbe92dfad86aa6e9301736f6b4342a3f5c", "size": 78146, "url": "https://libraries.minecraft.net/net/sf/jopt-simple/jopt-simple/5.0.4/jopt-simple-5.0.4.jar"}}, "name": "net.sf.jopt-simple:jopt-simple:5.0.4"}, {"downloads": {"artifact": {"path": "org/apache/commons/commons-compress/1.22/commons-compress-1.22.jar", "sha1": "691a8b4e6cf4248c3bc72c8b719337d5cb7359fa", "size": 1039712, "url": "https://libraries.minecraft.net/org/apache/commons/commons-compress/1.22/commons-compress-1.22.jar"}}, "name": "org.apache.commons:commons-compress:1.22"}, {"downloads": {"artifact": {"path": "org/apache/commons/commons-lang3/3.13.0/commons-lang3-3.13.0.jar", "sha1": "b7263237aa89c1f99b327197c41d0669707a462e", "size": 632267, "url": "https://libraries.minecraft.net/org/apache/commons/commons-lang3/3.13.0/commons-lang3-3.13.0.jar"}}, "name": "org.apache.commons:commons-lang3:3.13.0"}, {"downloads": {"artifact": {"path": "org/apache/httpcomponents/httpclient/4.5.13/httpclient-4.5.13.jar", "sha1": "e5f6cae5ca7ecaac1ec2827a9e2d65ae2869cada", "size": 780321, "url": "https://libraries.minecraft.net/org/apache/httpcomponents/httpclient/4.5.13/httpclient-4.5.13.jar"}}, "name": "org.apache.httpcomponents:httpclient:4.5.13"}, {"downloads": {"artifact": {"path": "org/apache/httpcomponents/httpcore/4.4.16/httpcore-4.4.16.jar", "sha1": "51cf043c87253c9f58b539c9f7e44c8894223850", "size": 327891, "url": "https://libraries.minecraft.net/org/apache/httpcomponents/httpcore/4.4.16/httpcore-4.4.16.jar"}}, "name": "org.apache.httpcomponents:httpcore:4.4.16"}, {"downloads": {"artifact": {"path": "org/apache/logging/log4j/log4j-api/2.19.0/log4j-api-2.19.0.jar", "sha1": "ea1b37f38c327596b216542bc636cfdc0b8036fa", "size": 317566, "url": "https://libraries.minecraft.net/org/apache/logging/log4j/log4j-api/2.19.0/log4j-api-2.19.0.jar"}}, "name": "org.apache.logging.log4j:log4j-api:2.19.0"}, {"downloads": {"artifact": {"path": "org/apache/logging/log4j/log4j-core/2.19.0/log4j-core-2.19.0.jar", "sha1": "3b6eeb4de4c49c0fe38a4ee27188ff5fee44d0bb", "size": 1864386, "url": "https://libraries.minecraft.net/org/apache/logging/log4j/log4j-core/2.19.0/log4j-core-2.19.0.jar"}}, "name": "org.apache.logging.log4j:log4j-core:2.19.0"}, {"downloads": {"artifact": {"path": "org/apache/logging/log4j/log4j-slf4j2-impl/2.19.0/log4j-slf4j2-impl-2.19.0.jar", "sha1": "5c04bfdd63ce9dceb2e284b81e96b6a70010ee10", "size": 27721, "url": "https://libraries.minecraft.net/org/apache/logging/log4j/log4j-slf4j2-impl/2.19.0/log4j-slf4j2-impl-2.19.0.jar"}}, "name": "org.apache.logging.log4j:log4j-slf4j2-impl:2.19.0"}, {"downloads": {"artifact": {"path": "org/joml/joml/1.10.5/joml-1.10.5.jar", "sha1": "22566d58af70ad3d72308bab63b8339906deb649", "size": 712082, "url": "https://libraries.minecraft.net/org/joml/joml/1.10.5/joml-1.10.5.jar"}}, "name": "org.joml:joml:1.10.5"}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-glfw/3.3.2/lwjgl-glfw-3.3.2.jar", "sha1": "757920418805fb90bfebb3d46b1d9e7669fca2eb", "size": 135828, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-glfw/3.3.2/lwjgl-glfw-3.3.2.jar"}}, "name": "org.lwjgl:lwjgl-glfw:3.3.2"}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-glfw/3.3.2/lwjgl-glfw-3.3.2-natives-linux.jar", "sha1": "0766bb0e8e829598b1c8052fd8173c62af741c52", "size": 115553, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-glfw/3.3.2/lwjgl-glfw-3.3.2-natives-linux.jar"}}, "name": "org.lwjgl:lwjgl-glfw:3.3.2:natives-linux", "rules": [{"action": "allow", "os": {"name": "linux"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-glfw/3.3.2/lwjgl-glfw-3.3.2-natives-macos.jar", "sha1": "8223ba1b757c43624f03b09ab9d9228c7f6db001", "size": 140627, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-glfw/3.3.2/lwjgl-glfw-3.3.2-natives-macos.jar"}}, "name": "org.lwjgl:lwjgl-glfw:3.3.2:natives-macos", "rules": [{"action": "allow", "os": {"name": "osx"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-glfw/3.3.2/lwjgl-glfw-3.3.2-natives-macos-arm64.jar", "sha1": "7e35644de1bf324ca9b7d52acd6e0b8d9a6da4ad", "size": 137535, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-glfw/3.3.2/lwjgl-glfw-3.3.2-natives-macos-arm64.jar"}}, "name": "org.lwjgl:lwjgl-glfw:3.3.2:natives-macos-arm64", "rules": [{"action": "allow", "os": {"name": "osx"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-glfw/3.3.2/lwjgl-glfw-3.3.2-natives-windows.jar", "sha1": "01251e3cb7e5d6159334cfb9244f789ce992f03b", "size": 165947, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-glfw/3.3.2/lwjgl-glfw-3.3.2-natives-windows.jar"}}, "name": "org.lwjgl:lwjgl-glfw:3.3.2:natives-windows", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-glfw/3.3.2/lwjgl-glfw-3.3.2-natives-windows-arm64.jar", "sha1": "e79c4857a887bd79ba78bdf2d422a7d333028a2d", "size": 141892, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-glfw/3.3.2/lwjgl-glfw-3.3.2-natives-windows-arm64.jar"}}, "name": "org.lwjgl:lwjgl-glfw:3.3.2:natives-windows-arm64", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-glfw/3.3.2/lwjgl-glfw-3.3.2-natives-windows-x86.jar", "sha1": "17e1f9ec031ef72c2f7825c38eeb3a79c4d8bb17", "size": 156550, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-glfw/3.3.2/lwjgl-glfw-3.3.2-natives-windows-x86.jar"}}, "name": "org.lwjgl:lwjgl-glfw:3.3.2:natives-windows-x86", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-jemalloc/3.3.2/lwjgl-jemalloc-3.3.2.jar", "sha1": "877e17e39ebcd58a9c956dc3b5b777813de0873a", "size": 43233, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-jemalloc/3.3.2/lwjgl-jemalloc-3.3.2.jar"}}, "name": "org.lwjgl:lwjgl-jemalloc:3.3.2"}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-jemalloc/3.3.2/lwjgl-jemalloc-3.3.2-natives-linux.jar", "sha1": "2bc33176cdfabf34a63df154b80914e8f433316b", "size": 204677, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-jemalloc/3.3.2/lwjgl-jemalloc-3.3.2-natives-linux.jar"}}, "name": "org.lwjgl:lwjgl-jemalloc:3.3.2:natives-linux", "rules": [{"action": "allow", "os": {"name": "linux"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-jemalloc/3.3.2/lwjgl-jemalloc-3.3.2-natives-macos.jar", "sha1": "959ea96ea27cd1ee45943123140fdb55c49e961f", "size": 153695, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-jemalloc/3.3.2/lwjgl-jemalloc-3.3.2-natives-macos.jar"}}, "name": "org.lwjgl:lwjgl-jemalloc:3.3.2:natives-macos", "rules": [{"action": "allow", "os": {"name": "osx"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-jemalloc/3.3.2/lwjgl-jemalloc-3.3.2-natives-macos-arm64.jar", "sha1": "fe75e1cad7ac1f7af4a071c4b83e39d11b65a1cc", "size": 142101, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-jemalloc/3.3.2/lwjgl-jemalloc-3.3.2-natives-macos-arm64.jar"}}, "name": "org.lwjgl:lwjgl-jemalloc:3.3.2:natives-macos-arm64", "rules": [{"action": "allow", "os": {"name": "osx"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-jemalloc/3.3.2/lwjgl-jemalloc-3.3.2-natives-windows.jar", "sha1": "db886c1f9e313c3fa2a25543b99ccd250d3f9fb5", "size": 180026, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-jemalloc/3.3.2/lwjgl-jemalloc-3.3.2-natives-windows.jar"}}, "name": "org.lwjgl:lwjgl-jemalloc:3.3.2:natives-windows", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-jemalloc/3.3.2/lwjgl-jemalloc-3.3.2-natives-windows-arm64.jar", "sha1": "598790de603c286dbc4068b27829eacc37592786", "size": 152780, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-jemalloc/3.3.2/lwjgl-jemalloc-3.3.2-natives-windows-arm64.jar"}}, "name": "org.lwjgl:lwjgl-jemalloc:3.3.2:natives-windows-arm64", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-jemalloc/3.3.2/lwjgl-jemalloc-3.3.2-natives-windows-x86.jar", "sha1": "9b07558f81a5d54dfaeb861bab3ccc86bb4477c9", "size": 148041, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-jemalloc/3.3.2/lwjgl-jemalloc-3.3.2-natives-windows-x86.jar"}}, "name": "org.lwjgl:lwjgl-jemalloc:3.3.2:natives-windows-x86", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-openal/3.3.2/lwjgl-openal-3.3.2.jar", "sha1": "ae5357ed6d934546d3533993ea84c0cfb75eed95", "size": 108230, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-openal/3.3.2/lwjgl-openal-3.3.2.jar"}}, "name": "org.lwjgl:lwjgl-openal:3.3.2"}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-openal/3.3.2/lwjgl-openal-3.3.2-natives-linux.jar", "sha1": "650981780b8bbfb3ce43657b22ec8e77bfb1f37a", "size": 579919, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-openal/3.3.2/lwjgl-openal-3.3.2-natives-linux.jar"}}, "name": "org.lwjgl:lwjgl-openal:3.3.2:natives-linux", "rules": [{"action": "allow", "os": {"name": "linux"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-openal/3.3.2/lwjgl-openal-3.3.2-natives-macos.jar", "sha1": "56fa16d15039142634e3a6d97d638b56679be821", "size": 515159, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-openal/3.3.2/lwjgl-openal-3.3.2-natives-macos.jar"}}, "name": "org.lwjgl:lwjgl-openal:3.3.2:natives-macos", "rules": [{"action": "allow", "os": {"name": "osx"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-openal/3.3.2/lwjgl-openal-3.3.2-natives-macos-arm64.jar", "sha1": "7b43d16069bdabc9ca0923ec8756a51c5d61cb75", "size": 467151, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-openal/3.3.2/lwjgl-openal-3.3.2-natives-macos-arm64.jar"}}, "name": "org.lwjgl:lwjgl-openal:3.3.2:natives-macos-arm64", "rules": [{"action": "allow", "os": {"name": "osx"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-openal/3.3.2/lwjgl-openal-3.3.2-natives-windows.jar", "sha1": "e74f299a602192faaf14b917632e4cbbb493c940", "size": 694908, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-openal/3.3.2/lwjgl-openal-3.3.2-natives-windows.jar"}}, "name": "org.lwjgl:lwjgl-openal:3.3.2:natives-windows", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-openal/3.3.2/lwjgl-openal-3.3.2-natives-windows-arm64.jar", "sha1": "545ddec7959007a78b6662d616e00dacf00e1c29", "size": 627059, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-openal/3.3.2/lwjgl-openal-3.3.2-natives-windows-arm64.jar"}}, "name": "org.lwjgl:lwjgl-openal:3.3.2:natives-windows-arm64", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-openal/3.3.2/lwjgl-openal-3.3.2-natives-windows-x86.jar", "sha1": "21fcb44d32ccf101017ec939fc740197677557d5", "size": 636273, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-openal/3.3.2/lwjgl-openal-3.3.2-natives-windows-x86.jar"}}, "name": "org.lwjgl:lwjgl-openal:3.3.2:natives-windows-x86", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-opengl/3.3.2/lwjgl-opengl-3.3.2.jar", "sha1": "ee8e95be0b438602038bc1f02dc5e3d011b1b216", "size": 928871, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-opengl/3.3.2/lwjgl-opengl-3.3.2.jar"}}, "name": "org.lwjgl:lwjgl-opengl:3.3.2"}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-opengl/3.3.2/lwjgl-opengl-3.3.2-natives-linux.jar", "sha1": "b8368430ef0d91a5acbc6fbfa47b20c3ec083aec", "size": 80464, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-opengl/3.3.2/lwjgl-opengl-3.3.2-natives-linux.jar"}}, "name": "org.lwjgl:lwjgl-opengl:3.3.2:natives-linux", "rules": [{"action": "allow", "os": {"name": "linux"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-opengl/3.3.2/lwjgl-opengl-3.3.2-natives-macos.jar", "sha1": "05141389ca737369f317b1288a570534c40db9cf", "size": 41485, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-opengl/3.3.2/lwjgl-opengl-3.3.2-natives-macos.jar"}}, "name": "org.lwjgl:lwjgl-opengl:3.3.2:natives-macos", "rules": [{"action": "allow", "os": {"name": "osx"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-opengl/3.3.2/lwjgl-opengl-3.3.2-natives-macos-arm64.jar", "sha1": "baadfd67936dcd7e0334dd3a9cf8dd13a0bcb009", "size": 42490, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-opengl/3.3.2/lwjgl-opengl-3.3.2-natives-macos-arm64.jar"}}, "name": "org.lwjgl:lwjgl-opengl:3.3.2:natives-macos-arm64", "rules": [{"action": "allow", "os": {"name": "osx"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-opengl/3.3.2/lwjgl-opengl-3.3.2-natives-windows.jar", "sha1": "83cd34469d4e0bc335bf74c7f62206530a9480bf", "size": 101530, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-opengl/3.3.2/lwjgl-opengl-3.3.2-natives-windows.jar"}}, "name": "org.lwjgl:lwjgl-opengl:3.3.2:natives-windows", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-opengl/3.3.2/lwjgl-opengl-3.3.2-natives-windows-arm64.jar", "sha1": "21df035bf03dbf5001f92291b24dc951da513481", "size": 83132, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-opengl/3.3.2/lwjgl-opengl-3.3.2-natives-windows-arm64.jar"}}, "name": "org.lwjgl:lwjgl-opengl:3.3.2:natives-windows-arm64", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-opengl/3.3.2/lwjgl-opengl-3.3.2-natives-windows-x86.jar", "sha1": "22fa4149159154b24f6c1bd46a342d4958a9fba1", "size": 88610, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-opengl/3.3.2/lwjgl-opengl-3.3.2-natives-windows-x86.jar"}}, "name": "org.lwjgl:lwjgl-opengl:3.3.2:natives-windows-x86", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-stb/3.3.2/lwjgl-stb-3.3.2.jar", "sha1": "a2550795014d622b686e9caac50b14baa87d2c70", "size": 118874, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-stb/3.3.2/lwjgl-stb-3.3.2.jar"}}, "name": "org.lwjgl:lwjgl-stb:3.3.2"}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-stb/3.3.2/lwjgl-stb-3.3.2-natives-linux.jar", "sha1": "5c987f43b342d722b54970159040af76f1c87403", "size": 231821, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-stb/3.3.2/lwjgl-stb-3.3.2-natives-linux.jar"}}, "name": "org.lwjgl:lwjgl-stb:3.3.2:natives-linux", "rules": [{"action": "allow", "os": {"name": "linux"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-stb/3.3.2/lwjgl-stb-3.3.2-natives-macos.jar", "sha1": "61614cb49cee0b95587893a36bd72f63ab815c82", "size": 216457, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-stb/3.3.2/lwjgl-stb-3.3.2-natives-macos.jar"}}, "name": "org.lwjgl:lwjgl-stb:3.3.2:natives-macos", "rules": [{"action": "allow", "os": {"name": "osx"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-stb/3.3.2/lwjgl-stb-3.3.2-natives-macos-arm64.jar", "sha1": "db49f0f76e8377520b625c688cd45aacb3dcdc9b", "size": 183630, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-stb/3.3.2/lwjgl-stb-3.3.2-natives-macos-arm64.jar"}}, "name": "org.lwjgl:lwjgl-stb:3.3.2:natives-macos-arm64", "rules": [{"action": "allow", "os": {"name": "osx"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-stb/3.3.2/lwjgl-stb-3.3.2-natives-windows.jar", "sha1": "1c4f4b8353bdb78c5264ab921436f03fc9aa1ba5", "size": 261137, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-stb/3.3.2/lwjgl-stb-3.3.2-natives-windows.jar"}}, "name": "org.lwjgl:lwjgl-stb:3.3.2:natives-windows", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-stb/3.3.2/lwjgl-stb-3.3.2-natives-windows-arm64.jar", "sha1": "c29df97c3cca97dc00d34e171936153764c9f78b", "size": 218460, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-stb/3.3.2/lwjgl-stb-3.3.2-natives-windows-arm64.jar"}}, "name": "org.lwjgl:lwjgl-stb:3.3.2:natives-windows-arm64", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-stb/3.3.2/lwjgl-stb-3.3.2-natives-windows-x86.jar", "sha1": "a0de7bde6722fa68d25ba6afbd7395508c53c730", "size": 227583, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-stb/3.3.2/lwjgl-stb-3.3.2-natives-windows-x86.jar"}}, "name": "org.lwjgl:lwjgl-stb:3.3.2:natives-windows-x86", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-tinyfd/3.3.2/lwjgl-tinyfd-3.3.2.jar", "sha1": "9f65c248dd77934105274fcf8351abb75b34327c", "size": 13404, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-tinyfd/3.3.2/lwjgl-tinyfd-3.3.2.jar"}}, "name": "org.lwjgl:lwjgl-tinyfd:3.3.2"}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-tinyfd/3.3.2/lwjgl-tinyfd-3.3.2-natives-linux.jar", "sha1": "05d27fc67172b3fcd8400d61a7cfb75da881f609", "size": 43961, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-tinyfd/3.3.2/lwjgl-tinyfd-3.3.2-natives-linux.jar"}}, "name": "org.lwjgl:lwjgl-tinyfd:3.3.2:natives-linux", "rules": [{"action": "allow", "os": {"name": "linux"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-tinyfd/3.3.2/lwjgl-tinyfd-3.3.2-natives-macos.jar", "sha1": "ce88dcda2fffe5d912de8ea1dabd68490a8f8471", "size": 45096, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-tinyfd/3.3.2/lwjgl-tinyfd-3.3.2-natives-macos.jar"}}, "name": "org.lwjgl:lwjgl-tinyfd:3.3.2:natives-macos", "rules": [{"action": "allow", "os": {"name": "osx"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-tinyfd/3.3.2/lwjgl-tinyfd-3.3.2-natives-macos-arm64.jar", "sha1": "0b4aa34d244c75bcbb78d6cdb0b41200348da330", "size": 41812, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-tinyfd/3.3.2/lwjgl-tinyfd-3.3.2-natives-macos-arm64.jar"}}, "name": "org.lwjgl:lwjgl-tinyfd:3.3.2:natives-macos-arm64", "rules": [{"action": "allow", "os": {"name": "osx"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-tinyfd/3.3.2/lwjgl-tinyfd-3.3.2-natives-windows.jar", "sha1": "54a93ed247d20007a6f579355263fdc2c030753a", "size": 130126, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-tinyfd/3.3.2/lwjgl-tinyfd-3.3.2-natives-windows.jar"}}, "name": "org.lwjgl:lwjgl-tinyfd:3.3.2:natives-windows", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-tinyfd/3.3.2/lwjgl-tinyfd-3.3.2-natives-windows-arm64.jar", "sha1": "500f5daa3b731ca282d4b90aeafda94c528d3e27", "size": 110758, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-tinyfd/3.3.2/lwjgl-tinyfd-3.3.2-natives-windows-arm64.jar"}}, "name": "org.lwjgl:lwjgl-tinyfd:3.3.2:natives-windows-arm64", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl-tinyfd/3.3.2/lwjgl-tinyfd-3.3.2-natives-windows-x86.jar", "sha1": "0c1dfa1c438e0262453e7bf625289540e5cbffb2", "size": 111596, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl-tinyfd/3.3.2/lwjgl-tinyfd-3.3.2-natives-windows-x86.jar"}}, "name": "org.lwjgl:lwjgl-tinyfd:3.3.2:natives-windows-x86", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl/3.3.2/lwjgl-3.3.2.jar", "sha1": "4421d94af68e35dcaa31737a6fc59136a1e61b94", "size": 786196, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl/3.3.2/lwjgl-3.3.2.jar"}}, "name": "org.lwjgl:lwjgl:3.3.2"}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl/3.3.2/lwjgl-3.3.2-natives-linux.jar", "sha1": "767684973f259d97e7dc66a125eb153986f177e7", "size": 114144, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl/3.3.2/lwjgl-3.3.2-natives-linux.jar"}}, "name": "org.lwjgl:lwjgl:3.3.2:natives-linux", "rules": [{"action": "allow", "os": {"name": "linux"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl/3.3.2/lwjgl-3.3.2-natives-macos.jar", "sha1": "3256dce7fa36d6b572afa5e5730f532cf987f3bf", "size": 60240, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl/3.3.2/lwjgl-3.3.2-natives-macos.jar"}}, "name": "org.lwjgl:lwjgl:3.3.2:natives-macos", "rules": [{"action": "allow", "os": {"name": "osx"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl/3.3.2/lwjgl-3.3.2-natives-macos-arm64.jar", "sha1": "319eea74a8829ce92fd54ce7c684b6b6557c05bb", "size": 48270, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl/3.3.2/lwjgl-3.3.2-natives-macos-arm64.jar"}}, "name": "org.lwjgl:lwjgl:3.3.2:natives-macos-arm64", "rules": [{"action": "allow", "os": {"name": "osx"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl/3.3.2/lwjgl-3.3.2-natives-windows.jar", "sha1": "a55169ced70ffcd15f2162daf4a9c968578f6cd5", "size": 164993, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl/3.3.2/lwjgl-3.3.2-natives-windows.jar"}}, "name": "org.lwjgl:lwjgl:3.3.2:natives-windows", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl/3.3.2/lwjgl-3.3.2-natives-windows-arm64.jar", "sha1": "d900e4678449ba97ff46fa64b22e0376bf8cd00e", "size": 133200, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl/3.3.2/lwjgl-3.3.2-natives-windows-arm64.jar"}}, "name": "org.lwjgl:lwjgl:3.3.2:natives-windows-arm64", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/lwjgl/lwjgl/3.3.2/lwjgl-3.3.2-natives-windows-x86.jar", "sha1": "ed495259b2c8f068794da0ffedfa7ae7c130b3c5", "size": 139365, "url": "https://libraries.minecraft.net/org/lwjgl/lwjgl/3.3.2/lwjgl-3.3.2-natives-windows-x86.jar"}}, "name": "org.lwjgl:lwjgl:3.3.2:natives-windows-x86", "rules": [{"action": "allow", "os": {"name": "windows"}}]}, {"downloads": {"artifact": {"path": "org/slf4j/slf4j-api/2.0.7/slf4j-api-2.0.7.jar", "sha1": "41eb7184ea9d556f23e18b5cb99cad1f8581fc00", "size": 63635, "url": "https://libraries.minecraft.net/org/slf4j/slf4j-api/2.0.7/slf4j-api-2.0.7.jar"}}, "name": "org.slf4j:slf4j-api:2.0.7"}], "logging": {"client": {"argument": "-Dlog4j.configurationFile=${path}", "file": {"id": "client-1.12.xml", "sha1": "bd65e7d2e3c237be76cfbef4c2405033d7f91521", "size": 888, "url": "https://piston-data.mojang.com/v1/objects/bd65e7d2e3c237be76cfbef4c2405033d7f91521/client-1.12.xml"}, "type": "log4j2-xml"}}, "mainClass": "net.minecraft.client.main.Main", "minimumLauncherVersion": 21, "releaseTime": "2023-11-08T13:59:58+00:00", "time": "2023-11-08T13:59:58+00:00", "type": "snapshot"}"#;
        let typed: Version = serde_json::from_str(raw).unwrap();
        println!("{:#?}", typed);
    }
}
