using System;

namespace OgsSgfDownloader.Models
{
    [Serializable] public class Ratings
    {
        public int Version { get; set; }
        public Overall Overall { get; set; }
    }
}