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
        }
    }
}
