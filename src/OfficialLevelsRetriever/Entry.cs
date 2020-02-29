using Spectrum.API;
using Spectrum.API.Interfaces.Plugins;
using Spectrum.API.Interfaces.Systems;
using System;
using System.Reflection;
using Harmony;
using UnityEngine;
using System.Collections;
using System.Collections.Generic;

namespace Corecii.NetworkDebugger
{
    public class Entry : IPlugin
    {
        public string FriendlyName => "OfficialLevelsRetriever";
        public string Author => "Corecii";
        public string Contact => "SteamID: Corecii; Discord: Corecii#3019";
        public static string PluginVersion = "Version C.1.0.0";

        public void Initialize(IManager manager, string ipcIdentifier)
        {
            try {
                var harmony = HarmonyInstance.Create("com.corecii.distance.officialLevelsRetriever");
                harmony.PatchAll(Assembly.GetExecutingAssembly());
            }
            catch (Exception e)
            {
                Console.WriteLine("Patching errors!\n" + e);
            }
        }

        public static T getComponent<T>() where T : MonoBehaviour
        {
            GameObject[] objs = UnityEngine.Object.FindObjectsOfType<GameObject>();
            foreach (GameObject tObj in objs)
            {
                T component = tObj.GetComponent<T>();
                if (component != null)
                    return component;
            }
            return null;
        }

        public static List<T> getComponents<T>() where T : MonoBehaviour
        {
            List<T> results = new List<T>();
            GameObject[] objs = UnityEngine.Object.FindObjectsOfType<GameObject>();
            foreach (GameObject tObj in objs)
            {
                T[] components = tObj.GetComponents<T>();
                results.AddRange(components);
            }
            return results;
        }

        [HarmonyPatch(typeof(LevelSetsManager))]
        [HarmonyPatch("Start")]
        class PatchLevels
        {
            static void Postfix()
            {
                try
                {
                    Console.WriteLine("Official Levels: [");
                    foreach (var level in G.Sys.LevelSets_.OfficialLevelInfosList_)
                    {
                        Console.WriteLine($"\"{level.fileNameWithoutExtension_}\",");
                    }
                    Console.WriteLine("]");
                }
                catch (Exception e)
                {
                    Console.WriteLine($"Error logging official levels: {e}");
                }
            }
        }

        public void Shutdown() { }
    }
}
