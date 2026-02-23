import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { readTextFile, writeTextFile, BaseDirectory, exists, mkdir } from "@tauri-apps/plugin-fs";
import { MediaItem } from "./types";

const VERIFIED_FILE = "verified.json";

function App() {
  const [items, setItems] = useState<MediaItem[]>([]);
  const [libraryPath, setLibraryPath] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [verifiedItems, setVerifiedItems] = useState<Set<string>>(new Set());
  const [isLoading, setIsLoading] = useState(false);

  // Load verified status on mount
  useEffect(() => {
    async function loadVerified() {
      try {
        const fileExists = await exists(VERIFIED_FILE, { baseDir: BaseDirectory.AppData });
        if (fileExists) {
          const content = await readTextFile(VERIFIED_FILE, { baseDir: BaseDirectory.AppData });
          const paths = JSON.parse(content) as string[];
          setVerifiedItems(new Set(paths));
        }
      } catch (err) {
        console.error("Failed to load verified items:", err);
      }
    }
    loadVerified();
  }, []);

  // Save verified status whenever it changes
  const saveVerified = useCallback(async (newVerified: Set<string>) => {
    try {
      const appDataExists = await exists("", { baseDir: BaseDirectory.AppData });
      if (!appDataExists) {
        await mkdir("", { baseDir: BaseDirectory.AppData, recursive: true });
      }

      const content = JSON.stringify(Array.from(newVerified));
      await writeTextFile(VERIFIED_FILE, content, { baseDir: BaseDirectory.AppData });
    } catch (err) {
      console.error("Failed to save verified items:", err);
    }
  }, []);

  const toggleVerified = (path: string) => {
    setVerifiedItems((prev) => {
      const next = new Set(prev);
      if (next.has(path)) {
        next.delete(path);
      } else {
        next.add(path);
      }
      saveVerified(next);
      return next;
    });
  };

  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Library Folder",
      });

      if (selected && typeof selected === "string") {
        setLibraryPath(selected);
        setSearchQuery(""); // Reset search on new folder
        setIsLoading(true);
        // Initial scan
        const results = await invoke<MediaItem[]>("scan_library", { path: selected });
        setItems(results);
        setIsLoading(false);
      }
    } catch (err) {
      console.error("Error selecting folder:", err);
      setIsLoading(false);
    }
  };

  // Debounced search
  useEffect(() => {
    const timer = setTimeout(async () => {
        // Only search if we have a library loaded (or if search logic allows searching empty? No, backend needs state)
        // Actually backend holds state. But if libraryPath is null, backend state is empty initially.
        try {
            const results = await invoke<MediaItem[]>("search", { query: searchQuery });
            setItems(results);
        } catch (err) {
            console.error("Search failed:", err);
        }
    }, 300);

    return () => clearTimeout(timer);
  }, [searchQuery]);

  return (
    <div className="min-h-screen bg-slate-900 text-slate-200 p-6 font-sans">
      <div className="max-w-7xl mx-auto space-y-6">

        {/* Header Section */}
        <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
            <div>
                <h1 className="text-3xl font-bold text-white tracking-tight">Media Stats Viewer</h1>
                <p className="text-slate-400 text-sm mt-1">
                    {libraryPath ? `Library: ${libraryPath}` : "No library selected"}
                </p>
            </div>

            <div className="flex items-center gap-4">
                <button
                    onClick={handleSelectFolder}
                    className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg shadow-lg transition-colors font-medium flex items-center gap-2"
                >
                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                    </svg>
                    Select Library
                </button>
            </div>
        </div>

        {/* Search Bar */}
        <div className="relative">
            <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <svg className="w-5 h-5 text-slate-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
            </div>
            <input
                type="text"
                placeholder="Search media..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="block w-full pl-10 pr-3 py-2.5 bg-slate-800 border border-slate-700 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent placeholder-slate-500 text-white shadow-sm transition-all"
            />
        </div>

        {/* Data Table */}
        <div className="bg-slate-800 rounded-xl shadow-xl overflow-hidden border border-slate-700">
            {isLoading ? (
                <div className="p-12 text-center text-slate-400">
                    <svg className="animate-spin h-8 w-8 mx-auto mb-4 text-blue-500" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                        <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                        <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                    </svg>
                    Scanning library...
                </div>
            ) : items.length === 0 ? (
                <div className="p-12 text-center text-slate-500">
                    {libraryPath ? "No items found matching your search." : "Select a library folder to begin."}
                </div>
            ) : (
                <div className="overflow-x-auto">
                    <table className="w-full text-left text-sm">
                        <thead className="bg-slate-700/50 text-slate-300 uppercase tracking-wider text-xs font-semibold">
                            <tr>
                                <th className="px-6 py-4 rounded-tl-lg">Verified</th>
                                <th className="px-6 py-4">Name</th>
                                <th className="px-6 py-4">Season</th>
                                <th className="px-6 py-4">Group</th>
                                <th className="px-6 py-4">Resolution</th>
                                <th className="px-6 py-4">Source</th>
                                <th className="px-6 py-4">Video</th>
                                <th className="px-6 py-4">Audio</th>
                                <th className="px-6 py-4 rounded-tr-lg">Avg Size (GB)</th>
                            </tr>
                        </thead>
                        <tbody className="divide-y divide-slate-700">
                            {items.map((item) => (
                                <tr key={item.path} className="hover:bg-slate-700/30 transition-colors">
                                    <td className="px-6 py-4">
                                        <input
                                            type="checkbox"
                                            checked={verifiedItems.has(item.path)}
                                            onChange={() => toggleVerified(item.path)}
                                            className="w-4 h-4 text-blue-600 bg-slate-700 border-slate-600 rounded focus:ring-blue-500 focus:ring-offset-slate-800"
                                        />
                                    </td>
                                    <td className="px-6 py-4 font-medium text-white">{item.name}</td>
                                    <td className="px-6 py-4 text-slate-300">{item.season || "-"}</td>
                                    <td className="px-6 py-4 text-slate-400">{item.group}</td>
                                    <td className="px-6 py-4">
                                        <span className="px-2 py-1 rounded-full bg-slate-700 text-xs font-medium text-slate-300 border border-slate-600">
                                            {item.resolution}
                                        </span>
                                    </td>
                                    <td className="px-6 py-4 text-slate-300">{item.source}</td>
                                    <td className="px-6 py-4 text-slate-300">{item.video_codec}</td>
                                    <td className="px-6 py-4 text-slate-300">{item.audio_codec}</td>
                                    <td className="px-6 py-4 text-slate-300 font-mono">
                                        {item.avg_size_gb.toFixed(2)}
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            )}
        </div>
      </div>
    </div>
  );
}

export default App;
