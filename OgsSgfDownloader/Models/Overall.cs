using System;

namespace OgsSgfDownloader.Models
{
    [Serializable] public class Overall
    {
        public double Rating { get; set; }
        public double Deviation { get; set; }
        public double Volatility { get; set; }
    }
}