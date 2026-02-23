export interface MediaItem {
  name: string;
  group: string;
  resolution: string;
  source: string;
  video_codec: string;
  audio_codec: string;
  season?: string;
  path: string;
  is_airing: boolean;
  avg_size_gb: number;
}
