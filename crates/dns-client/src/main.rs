use dns::protocol::answer::Answer;

fn main() {
    let mut args = std::env::args();
    args.next();
    let domain = args.next().expect("Please specify a domain name");
    let dns_server = args.next().unwrap_or_else(|| "1.1.1.1".into());

    println!("Resolving {domain} via DNS {dns_server}\n\n");

    let (answers, _) = dns::resolver::resolve_domain(&domain, &dns_server, None, None)
        .expect("Error resolving DNS records");

    for answer in answers {
        match answer {
            Answer::A { meta, ipv4 } => println!("A\t{meta:?} - {ipv4}"),
            Answer::CNAME { meta, cname } => println!("CNAME\t{meta:?} - {cname}"),
            Answer::AAAA { meta, ipv6 } => println!("CNAME\t{meta:?} - {ipv6}"),
            Answer::NS { ns, meta } => println!("CNAME\t{meta:?} - {ns}"),
            Answer::MB { domain_name, meta } => println!("CNAME\t{meta:?} - {domain_name}"),
            Answer::MX {
                preference,
                exchange,
                meta,
            } => println!("CNAME\t{meta:?} - {exchange} ({preference})"),
            Answer::PTR { domain_name, meta } => println!("CNAME\t{meta:?} - {domain_name}"),
            Answer::SOA {
                meta,
                mname,
                rname,
                serial: _,
                refresh: _,
                retry: _,
                expire: _,
                minimum: _,
            } => println!("CNAME\t{meta:?} - {mname} - {rname}"),
        }
    }
}
