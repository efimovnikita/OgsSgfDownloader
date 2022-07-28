using System;

namespace OgsSgfDownloader.Models
{
    [Serializable] public class HistoricalRatings
    {
        public Black Black { get; set; }
        public White White { get; set; }
    }
}