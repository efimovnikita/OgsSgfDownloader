using System;
using System.Collections.Generic;

namespace OgsSgfDownloader.Models
{
    [Serializable] public class Root
    {
        public int Count { get; set; }
        public string Next { get; set; }
        public object Previous { get; set; }
        public List<Result> Results { get; set; }
    }
}