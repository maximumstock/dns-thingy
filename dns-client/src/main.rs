fn main() {
    let mut args = std::env::args();
    args.next();
    let domain = args.next().expect("Please specify a domain name");
    let dns_server = args.next().unwrap_or_else(|| "1.1.1.1".into());

    println!("Resolving {} via DNS {}\n\n", domain, dns_server);

    let answers =
        dns::resolver::resolve(&domain, &dns_server).expect("Error resolving DNS records");

    for answer in answers {
        match answer {
            dns::dns::Answer::A { meta, ipv4 } => println!("A\t{:?} - {}", meta, ipv4),
            dns::dns::Answer::CNAME { meta, cname } => println!("CNAME\t{:?} - {}", meta, cname),
        }
    }
}
