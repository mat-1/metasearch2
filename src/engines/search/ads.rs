use rand::prelude::SliceRandom;

use crate::engines::{EngineResponse, EngineSearchResult, RequestResponse};

pub fn get_ad_search_results() -> Vec<EngineSearchResult> {
    let mut results = vec![];
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/sspa/click".to_string(),
        title: "luomart Girls Cat Birthstone Necklaces Jewelry,Silver Plated Kitty Dog Pendant Gifts Set for Women Boys Men".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/sspa/click".to_string(),
        title: "Anti Anxiety Round Fluffy Plush Faux Fur Warm Washable Dog Bed & Cat Bed, Original Bed for Small Medium Large Pets,Used to Relieve Joints and Improve Sleep（20\"/24\"/27''） (20\", Light Green)".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/sspa/click".to_string(),
        title: "Busters Calming Hemp Oil, Enriched with Melatonin for Dogs, Cats, Pets, Sleep Aid, Natural Anxiety Relief, Ideal Omega Ratio, Adrenal and Cushings Support".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Yaheetech-Tower-Furniture-Scratch-Kittens/dp/B0794T79KM"
            .to_string(),
        title:
            "Yaheetech 54in Cat Tree Tower Condo Furniture Scratch Post for Kittens Pet House Play"
                .to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Shedding-Brushes-DONOTU-Cleaning-Grooming/dp/B0BC5CNP3S".to_string(),
        title: "Cat Brush for Shedding Long or Short Haired Cats, Cat Brushes for Indoor Cats, DONOTU Self Cleaning Slicker Brush for Large Medium Small Dogs, Pets Grooming Tool, Removes Mats, Tangles and Loose Fur".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Vital-Essentials-Freeze-Dried-Treats-Minnows/dp/B0BWBK16NH"
            .to_string(),
        title: "Vital Essentials Freeze-Dried Raw Cat Treats, Minnows Treats, 0.5 oz".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/INABA-Grain-Free-Lickable-Squeezable-Vitamin/dp/B0B52FX4WX".to_string(),
        title: "INABA Churu Cat Treats, Grain-Free, Lickable, Squeezable Creamy Purée Cat Treat/Topper with Vitamin E & Taurine, 0.5 Ounces Each Tube, 20 Tubes, Seafood Variety Box".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/AIERSA-Launcher-Shooter-Shooting-Interactive/dp/B0CGHVTXFF".to_string(),
        title: "AIERSA Cat Toy Ball Launcher Gun,Cat Fetch Toy Gun Shooter, Plush Ball Shooting Gun with 20Pcs Pom Pom Balls, Toys Interactive for Indoor Cats".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/MILIFUN-Double-Automatic-Waterer-Bottle/dp/B088WHK7F3".to_string(),
        title: "MILIFUN Double Dog Cat Bowls - Pets Water and Food Bowl Set, 15°Tilted Water and Food Bowl Set with Automatic Waterer Bottle for Small or Medium Size Dogs Cats".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Temptations-Mixups-Treats-Catnip-Holiday/dp/B00OLSARS2"
            .to_string(),
        title: "TEMPTATIONS MIXUPS Crunchy and Soft Cat Treats Catnip Fever Flavor, 16 oz. Tub"
            .to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/sspa/click".to_string(),
        title: "Amazon Basics Cat Pad Refills for Litter Box, Unscented, Pack of 60, Purple"
            .to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/sspa/click".to_string(),
        title: "Lesure Cat Bed for Indoor Cats - Fluffy Large Cat House for Kitten and Small Pet, Enclosed Cat Cave with Removable Washable Sherpa Cover, Cute Cat Hideaway with Non-Slip Bottom, 16\" x 16\" x 16\", Green".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/sspa/click".to_string(),
        title: "Cat Blanket Gifts for Women Cat Gifts for Cat Lovers Soft Flannel Kawaii Cat Throw Blanket for Kids Adults 50\"x40\"".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/sspa/click".to_string(),
        title: "Wireless Cat Water Fountain, 2L/67oz Automatic Stainless Pet Water Fountain Battery Operated, 270°Wide Sensor, 3 Replacement Filters, Dog Water Dispenser for Multiple Pets, BPA-Free, Silver".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Holder-Stopper-Adjustable-Measuring-Install/dp/B0BLSVRCPF".to_string(),
        title: "Cat Door Latch, 2 Pcs Flex Latch Cat Door Holder, Cat Door Stopper to Keep Dog Out of Litter Boxes and Food, 5 Adjustable Size Strap 2.5-6\" Wide, No Measuring, Easy to Install, White".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/gp/bestsellers/pet-supplies/2975310011".to_string(),
        title: "SmartyKat Catnip for Cats & Kittens, Shaker Canister - 2 Ounces".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/gp/bestsellers/pet-supplies/2975254011".to_string(),
        title: "rabbitgoo Cat Harness and Leash for Walking, Escape Proof Soft Adjustable Vest Harnesses for Cats, Easy Control Breathable Reflective Strips Jacket, Black, XS".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Cat-Bonnet-Scratcher-Reversible-Scratching/dp/B0CM1GQHV5".to_string(),
        title: "Cat Scratcher with 3 Reversible Scratching Pads Durable Cardboard Cat Scratching Board Helps Keep Your Cat's Claws Healthy with Unique Koi Fish Design".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/sspa/click".to_string(),
        title: "Family Wall Art - Inspirational Quotes Scripture Bible Verse Wall Art- Catholic Christian Gifts for Women - Religious Home Decor - Spiritual Wall Art - Positive Saying - God Poster - Cat Wall Art 8x10".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/sspa/click".to_string(),
        title: "Stainless Steel Litter Box High Sides, Extra Large XL Cat Litter Box Enclosure for Big Multiple Cats with Lid, Metal Litter Pan Tray, Non-Sticky, Include Litter Mat & Scoop".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/sspa/click".to_string(),
        title: "POPLYKE Cat/Family Cat Necklace for Women Girls 925 Sterling Silver Celtic Moon Cat Wiccan Jewelry Giftsfor Wife".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/sspa/click".to_string(),
        title: "Cat Blanket Super Soft Flannel Throw Blanket Just a Girl Who Loves cat Blankets Cat Gifts for Cat Lovers Cozy Lightweight Blankets for Women Kids Adults 50\"X 40\"".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Polarduck-Cat-Litter-Trapping-Mat-Honeycomb-Material-Washable/dp/B07TYJC6NV".to_string(),
        title: "Conlun Cat Litter Mat Cat Litter Trapping Mat, Honeycomb Double Layer Design, Urine and Water Proof Material, Scatter Control, Less Waste，Easier to Clean,Washable".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/gp/bestsellers/pet-supplies/2975246011".to_string(),
        title: "Bedsure Cat Beds for Indoor Cats - Large Cat Cave for Pet Cat House with Fluffy Ball Hanging and Scratch Pad, Foldable Cat Hideaway,16.5x16.5x13 inches, Grey".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/FELINE-GREENIES-Dental-Roasted-Chicken/dp/B0828WNJXC".to_string(),
        title: "Greenies Feline Adult Natural Dental Cat Treats, Oven Roasted Chicken Flavor, 9.75 oz. Tub".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/gp/bestsellers/pet-supplies/3024127011".to_string(),
        title: "Self Warming Cat Bed Self Heating Cat Dog Mat 24 x 18 inch Extra Warm Thermal Pet Pad for Indoor Outdoor Pets with Removable Cover Non-Slip Bottom Washable".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/sspa/click".to_string(),
        title: "Cat Dog Paw Pad Balm Stick (2.4 oz) | Natural Lick Safe Dog Paw Blam Protector, Soother & Moisturizer for Cracked Dry & Damaged Paws, Nose & Elbows | Snout Soother for Dogs".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Cowjag-Adjustable-Patterns-Recharge-Interactive/dp/B0BGBHXSWH".to_string(),
        title: "Cowjag Cat Toys, Laser Pointer with 5 Adjustable Patterns, USB Recharge Laser, Long Range and 3 Modes Training Chaser Interactive Toy, Dog Laser Toy".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/SINROBO-Catnip-Silvervine-Healthy-Cleaning/dp/B0BVYS2VLP".to_string(),
        title: "Catnip Ball for Cats Wall, 4 Pack, Silvervine Balls, Edible Toys, Lick Safe Healthy Kitten Chew & Teeth Cleaning Dental Toys, Wall Treats (Grey)".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Purina-Fancy-Feast-Variety-Collection/dp/B01ALT2IRM".to_string(),
        title: "Fancy Feast Poultry and Beef Feast Classic Pate Collection Grain Free Wet Cat Food Variety Pack - (Pack of 30) 3 oz. Cans".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Cats-Pride-Ultimate-Scented-Multi-Cat/dp/B01C600O0W".to_string(),
        title: "Cat's Pride Premium Lightweight Clumping Litter: Pure & Fresh - Up to 10 Days of Powerful Odor Control - Multi-Cat, Scented, 10 Pounds".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/TEMPTATIONS-Classic-Favorites-Variety-Pouches/dp/B008COEV4M".to_string(),
        title: "TEMPTATIONS Classic Crunchy and Soft Cat Treats Feline Favorite Variety Pack, 3 oz. Pouches,4 Count".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/gp/bestsellers/pet-supplies/2975263011".to_string(),
        title: "Veken 95oz/2.8L Pet Fountain, Automatic Cat Water Fountain Dog Water Dispenser with Replacement Filters for Cats, Dogs, Multiple Pets (Grey, Plastic)".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/BestPet-54in-Multi-Level-Furniture-Scratching/dp/B0BBGCD3QJ".to_string(),
        title: "BestPet 54in Multi-Level Cat Tree Tower Furniture Activity Center with Scratching Posts, Toys and Condo for Indoor Kittens, Beige".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Bedsure-Calming-Cat-Small-Cats/dp/B08PQQ2FHQ".to_string(),
        title: "Bedsure Calming Cat Beds for Indoor Cats - Small Cat Bed Washable 20 inches, Anti-Slip Round Fluffy Plush Faux Fur Pet Bed, Fits up to 15 lbs Pets, Camel".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/gp/bestsellers/pet-supplies/6514357011".to_string(),
        title: "SHEBA PERFECT PORTIONS Paté Adult Wet Cat Food Trays (24 Count, 48 Servings), Signature Seafood Entrée, Easy Peel Twin-Pack Trays".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/KSIIA-Sisal-Covered-Scratching-Multi-Level-Perches/dp/B0CNKFLXWS".to_string(),
        title: "KSIIA Cat Tree for Indoor Cats 38 inch Cat Tower with Sisal-Covered Scratching Post and Multi-Level Perches Kittens Cozy Cat Condo, Grey".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Fresh-Step-Multi-Cat-Febreze-Shield/dp/B000VDR8LA".to_string(),
        title: "Fresh Step Clumping Cat Litter, Multi-Cat Odor Control, 14 lbs".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Fashions-Talk-Variety-Kitty-Pieces/dp/B01AHM6P18".to_string(),
        title: "Cat Toys Variety Pack for Kitty 20 Pieces".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Zakkart-Perch-Window-Sill-Bolster/dp/B0CBTXSBFW".to_string(),
        title: "Cat Perch for Window Sill with Bolster - Orthopedic Hammock Design with Premium Hardwood & Robust Metal Frame - Cat Window Seat for Large Cats and Kittens - Nartural Color Wood with Gray Bed".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Catcan-Breathable-Adjustable-Grooming-Polyester/dp/B099WQ5SSF".to_string(),
        title: "Cat Bathing Bag, Breathable Mesh Anti Scratch Adjustable Cat Grooming Bag for Nail Trimming, Bathing Polyester Soft Cat Washing Bag (Blue-Orange)".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Purina-Fancy-Feast-Grilled-Collection/dp/B0010B6IFY".to_string(),
        title: "Purina Fancy Feast Grilled Wet Cat Food Poultry and Beef Collection Wet Cat Food Variety Pack - (Pack of 24) 3 oz. Cans".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/gp/bestsellers/pet-supplies/2975298011".to_string(),
        title: "Dr. Elsey's Premium Clumping Cat Litter - Ultra - 99.9% Dust-Free, Low Tracking, Hard Clumping, Superior Odor Control, Unscented & Natural Ingredients".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Purina-Tender-Selects-Blend-Salmon/dp/B001VIWGFW".to_string(),
        title:
            "Purina ONE Natural Dry Cat Food, Tender Selects Blend With Real Salmon - 3.5 lb. Bag"
                .to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Catstages-Slow-Feeder-Bowl-Blue/dp/B0B5W28K5X".to_string(),
        title: "Catstages Kitty Slow Feeder Cat Bowl, Blu".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/INABA-Grain-Free-Lickable-Squeezable-Vitamin/dp/B0B52F6Q1M".to_string(),
        title: "INABA Churu Cat Treats, Lickable, Squeezable Creamy Purée Cat Treat with Green Tea Extract & Taurine, 0.5 Ounces Each Tube, 20 Tubes, Tuna & Seafood Variety Box".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Portable-Hairball-Epilator-Removing-Furniture/dp/B09LH4BWCY".to_string(),
        title: "Pet Hair Remover, Dog Cat Hair Remover, Lint Cleaner Pro, Fur Removal Tool, Portable Carpet Scraper, Clothes Fuzz Rollers Hairball Shaver Brush for Carpets, Car Mat, Couch, Pet Bed, Furniture & Rug".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/gp/bestsellers/pet-supplies/2975249011".to_string(),
        title: "AMOSIJOY Cordless Cat Window Perch, Cat Hammock for Window with 4 Strong Suction Cups, Solid Metal Frame and Soft Cover, Cat Beds for Indoor Cats".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Fresh-Step-Outstretch-Concentrated-Freshness/dp/B093QNWC6T".to_string(),
        title: "Fresh Step Outstretch, Clumping Cat Litter, Advanced, Extra Large, 32 Pounds total (2 Pack of 16lb Boxes)".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/YVE-LIFE-Generation-Rechargeable-Interactive/dp/B0C7GMX4FT".to_string(),
        title: "Laser Cat Toys for Indoor Cats,The 4th Generation Real Random Trajectory Motion Activated Rechargeable Automatic Cat Laser Toy,Interactive Cat Toys for Bored Indoor Adult Cats/Kittens/Dogs".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Legendog-Catnip-Resistant-Cartoon-Teething/dp/B07QZQWDGY".to_string(),
        title: "5Pcs Bite Resistant Catnip Toy for Cats,Catnip Filled Cartoon Mice Cat Teething Chew Toy".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Hatphile-Panel-Embroidery-Baseball-Black/dp/B089LN9L44".to_string(),
        title: "Cat Mom & Dad Hats for Proud Cat Parents | for Men & Women | Embroidered Text - Adjustable Fit - 100% Cotton".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/AILUKI-Assortments-Variety-Including-Colorful/dp/B07FY82YPP".to_string(),
        title: "AILUKI 31 PCS Cat Toys Kitten Toys Assortments,Variety Catnip Toy Set Including 2 Way Tunnel,Cat Feather Teaser,Catnip Fish,Mice,Colorful Balls and Bells for Cat,Puppy,Kitty".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/gp/bestsellers/pet-supplies/3024133011".to_string(),
        title: "Henkelion Cat, Dog Carrier for Small Medium Cats Puppies up to 15 Lbs, TSA Airline Approved Carrier Soft Sided, Collapsible Travel Puppy Carrier - Grey".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Temptations-Jumbo-Treats-Tempting-Flavor/dp/B07QVTZ78G"
            .to_string(),
        title:
            "TEMPTATIONS Jumbo Stuff Crunchy and Soft Cat Treats, Tempting Tuna Flavor, 14 oz. Tub"
                .to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Blue-Buffalo-Bursts-Crunchy-Seafood/dp/B0832578WQ".to_string(),
        title: "Blue Buffalo Bursts Crunchy Cat Treats, Seafood 5-oz Bag".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/TEMPTATIONS-Classic-Treats-Tempting-Flavor/dp/B01BMFDS0K"
            .to_string(),
        title: "TEMPTATIONS Classic Crunchy and Soft Cat Treats Tempting Tuna Flavor, 16 oz. Tub"
            .to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Delectables-Senior-Years-Lickable-Treats/dp/B00T76GM64".to_string(),
        title: "Hartz Delectables Stew Senior Lickable Wet Cat Treats, Multiple Flavors 1.4 Ounce (Pack of 12)".to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/TEMPTATIONS-Classic-Treats-Savory-Salmon/dp/B00OLSARU0"
            .to_string(),
        title: "TEMPTATIONS Classic Crunchy and Soft Cat Treats Savory Salmon Flavor, 16 oz. Tub"
            .to_string(),
        description: "".to_string(),
    });
    results.push(EngineSearchResult {
        url: "https://www.amazon.com/Lives-Seafood-Poultry-Favorites-Variety/dp/B01GX0SNHC"
            .to_string(),
        title: "9Lives Seafood & Poultry Favorites Wet Cat Food Variety 5.5 Ounce Can (Pack of 24)"
            .to_string(),
        description: "".to_string(),
    });

    results.shuffle(&mut rand::thread_rng());
    results.truncate(10);
    results
}

pub fn request(query: &str) -> RequestResponse {
    let _ = query; // we care about money, not being useful

    RequestResponse::Instant(EngineResponse {
        search_results: get_ad_search_results(),
        featured_snippet: None,
        answer_html: None,
        infobox_html: None,
    })
}

pub fn parse_response(_: &str) -> eyre::Result<EngineResponse> {
    unreachable!()
}
