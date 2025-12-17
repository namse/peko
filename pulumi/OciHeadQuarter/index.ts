import * as pulumi from "@pulumi/pulumi";
import * as oci from "@pulumi/oci";
import * as random from "@pulumi/random";
import * as command from "@pulumi/command";
import * as k8s from "@pulumi/kubernetes";
import * as docker from "@pulumi/docker";
import * as yaml from "js-yaml";
import { OciWorkerInfraEnvs } from "../OciComputeWorker";
import { hqGrafana } from "./grafana";

export interface OciHeadQuarterArgs {
  region: pulumi.Input<string>;
  compartmentId: pulumi.Input<string>;
  vcnId: pulumi.Input<string>;
  ipv6cidrBlocks: pulumi.Input<string[]>;
  ociWorkerInfraEnvs: pulumi.Input<OciWorkerInfraEnvs>;
  grafanaRegion: pulumi.Input<string>;
  grafanaSlug: pulumi.Input<string>;
}

export class OciHeadQuarter extends pulumi.ComponentResource {
  constructor(
    name: string,
    args: OciHeadQuarterArgs,
    opts: pulumi.ComponentResourceOptions
  ) {
    const resourceInputs = { ...args };
    delete (resourceInputs as any).ociWorkerInfraEnvs;
    super("pkg:index:oci-head-quarter", name, resourceInputs, opts);

    const { region, compartmentId, vcnId, ociWorkerInfraEnvs } = args;

    const nameSuffix8 = new random.RandomString(
      "name-suffix-8",
      {
        length: 8,
        special: false,
        upper: false,
      },
      { parent: this }
    ).result;

    const internetGateway = new oci.core.InternetGateway(
      "igw",
      {
        compartmentId,
        vcnId,
      },
      { parent: this }
    );

    const routeTable = new oci.core.RouteTable(
      "route-table",
      {
        compartmentId,
        vcnId,
        routeRules: [
          {
            destination: "::/0",
            destinationType: "CIDR_BLOCK",
            networkEntityId: internetGateway.id,
          },
          {
            destination: "0.0.0.0/0",
            destinationType: "CIDR_BLOCK",
            networkEntityId: internetGateway.id,
          },
        ],
      },
      { parent: this }
    );

    const myIp = new command.local.Command(
      "my-ip",
      {
        create: "curl -s ifconfig.co",
      },
      { parent: this }
    ).stdout;

    const securityList = new oci.core.SecurityList(
      "security-list",
      {
        compartmentId,
        vcnId,
        egressSecurityRules: [
          {
            destination: "0.0.0.0/0",
            destinationType: "CIDR_BLOCK",
            protocol: "all",
            stateless: false,
          },
          {
            destination: "::/0",
            destinationType: "CIDR_BLOCK",
            protocol: "all",
            stateless: false,
          },
        ],
        ingressSecurityRules: [
          {
            source: myIp.apply((ip) => `${ip}/32`),
            protocol: "all",
            stateless: false,
          },
          {
            source: "10.0.0.0/16",
            protocol: "all",
            stateless: false,
          },
        ],
      },
      { parent: this }
    );

    const regionalSubnet = new oci.core.Subnet(
      "regional-subnet",
      {
        displayName: "fn0-hq-regional-subnet",
        compartmentId,
        vcnId,
        ipv4cidrBlocks: ["10.0.2.0/24"],
        routeTableId: routeTable.id,
        securityListIds: [securityList.id],
      },
      { parent: this, deleteBeforeReplace: true }
    );

    const ad1 = pulumi.all([compartmentId]).apply(([compartmentId]) =>
      oci.identity
        .getAvailabilityDomain({
          adNumber: 1,
          compartmentId,
        })
        .then((x) => x.name)
    );

    const clusterOptions = pulumi
      .all([compartmentId])
      .apply(([compartmentId]) => {
        return oci.containerengine.getClusterOption({
          clusterOptionId: "all",
          compartmentId,
        });
      });

    const kubernetesVersion = clusterOptions.apply((options) => {
      return options.kubernetesVersions.sort().pop()!;
    });

    const cluster = new oci.containerengine.Cluster(
      "cluster",
      {
        compartmentId,
        kubernetesVersion,
        clusterPodNetworkOptions: [
          {
            cniType: "OCI_VCN_IP_NATIVE",
          },
        ],
        endpointConfig: {
          isPublicIpEnabled: true,
          subnetId: regionalSubnet.id,
        },
        vcnId,
        name: pulumi.interpolate`fn0-${nameSuffix8}`,
      },
      { parent: this, deleteBeforeReplace: true }
    );

    const poolOptions = pulumi
      .all([compartmentId, kubernetesVersion])
      .apply(([compartmentId, kubernetesVersion]) => {
        return oci.containerengine.getNodePoolOption({
          compartmentId,
          nodePoolOptionId: "all",
          nodePoolK8sVersion: kubernetesVersion,
        });
      });

    const imageId = poolOptions.apply(
      (options) =>
        options.sources
          .filter((x) => x.sourceName.includes("-aarch64-"))
          .sort((a, b) => a.sourceName.localeCompare(b.sourceName))
          .pop()!.imageId
    );

    const nodePool = new oci.containerengine.NodePool(
      "node-pool",
      {
        compartmentId,
        clusterId: cluster.id,
        kubernetesVersion,
        name: pulumi.interpolate`fn0-nodepool-${nameSuffix8}`,
        nodeShape: "VM.Standard.A1.Flex",
        nodeShapeConfig: {
          ocpus: 1,
          memoryInGbs: 6,
        },
        nodeConfigDetails: {
          size: 1,
          placementConfigs: [
            {
              availabilityDomain: ad1,
              subnetId: regionalSubnet.id,
            },
          ],
          nodePoolPodNetworkOptionDetails: {
            cniType: "OCI_VCN_IP_NATIVE",
            podSubnetIds: [regionalSubnet.id],
          },
        },
        nodeSourceDetails: {
          imageId,
          sourceType: "IMAGE",
        },
      },
      { parent: this }
    );

    const { hqImage } = deployDocker(this);

    const config = new pulumi.Config("oci");
    const tenancyOcid = config.require("tenancyOcid");
    const userOcid = config.require("userOcid");
    const fingerprint = config.require("fingerprint");
    const privateKey = config.require("privateKey");

    const kubeconfig = pulumi
      .all([cluster.id, region])
      .apply(([clusterId, region]) =>
        oci.containerengine
          .getClusterKubeConfig({
            clusterId,
          })
          .then((kc) => {
            const content = yaml.load(kc.content) as {
              users: {
                user: {
                  exec: {
                    env: { name: string; value: string }[];
                  };
                };
              }[];
            };
            const { env } = content.users[0].user.exec;
            env.push(
              { name: "OCI_CLI_AUTH", value: "api_key" },
              { name: "OCI_CLI_REGION", value: region },
              { name: "OCI_CLI_USER", value: userOcid },
              { name: "OCI_CLI_TENANCY", value: tenancyOcid },
              { name: "OCI_CLI_FINGERPRINT", value: fingerprint },
              { name: "OCI_CLI_KEY_CONTENT", value: privateKey }
            );
            const result = yaml.dump(content);
            return result;
          })
      );

    const k8sProvider = new k8s.Provider(
      "oke-k8s-provider",
      {
        kubeconfig,
      },
      { parent: this, dependsOn: [nodePool] }
    );

    hqGrafana(this, {
      regionSlug: args.grafanaRegion,
      slug: args.grafanaSlug,
      k8sProvider: k8sProvider,
      suffix: nameSuffix8,
    });

    // const appLabels = { app: "hq" };

    // const deployment = new k8s.apps.v1.Deployment(
    //   "hq-deployment",
    //   {
    //     metadata: { labels: appLabels },
    //     spec: {
    //       replicas: 1,
    //       selector: { matchLabels: appLabels },
    //       template: {
    //         metadata: { labels: appLabels },
    //         spec: {
    //           containers: [
    //             {
    //               name: appLabels.app,
    //               image: hqImage.imageName,
    //               ports: [{ containerPort: 8080 }],
    //               livenessProbe: {
    //                 httpGet: {
    //                   path: "/health",
    //                   port: 8080,
    //                 },
    //                 initialDelaySeconds: 15,
    //                 periodSeconds: 5,
    //                 timeoutSeconds: 5,
    //                 failureThreshold: 3,
    //               },
    //               env: pulumi
    //                 .all([ociWorkerInfraEnvs])
    //                 .apply(([ociWorkerInfraEnvs]) => [
    //                   ...Object.entries(ociWorkerInfraEnvs).map(
    //                     ([name, value]) => ({
    //                       name,
    //                       value,
    //                     })
    //                   ),
    //                   {
    //                     name: "OTLP_ENDPOINT",
    //                     value: "http://alloy-service.monitoring:4317",
    //                   },
    //                 ]),
    //             },
    //           ],
    //         },
    //       },
    //     },
    //   },
    //   { provider: k8sProvider, parent: this }
    // );

    function deployDocker(parent: pulumi.Resource) {
      const repo = new oci.artifacts.ContainerRepository(
        "hq-repo",
        {
          compartmentId,
          displayName: pulumi.interpolate`hq-repo-${nameSuffix8}`,
          isPublic: true,
        },
        { parent, retainOnDelete: false }
      );

      const user = new oci.identity.User(
        "hq-user",
        {
          name: pulumi.interpolate`hq-user-${nameSuffix8}`,
          description: "User for HQ deployment",
        },
        { parent }
      );
      const dockerGroup = new oci.identity.Group(
        "hq-docker-pusher-group",
        {
          name: pulumi.interpolate`hq-docker-pushers-${nameSuffix8}`,
          description: "Group allowed to push to OCIR",
        },
        { parent }
      );
      new oci.identity.UserGroupMembership(
        "hq-membership",
        {
          userId: user.id,
          groupId: dockerGroup.id,
        },
        { parent }
      );
      new oci.identity.Policy(
        "ocir-push-policy",
        {
          compartmentId,
          name: pulumi.interpolate`allow-docker-push-${nameSuffix8}`,
          description: "Policy to allow docker pushers to manage repos",
          statements: [
            pulumi.interpolate`Allow group ${dockerGroup.name} to manage repos in compartment id ${compartmentId}`,
          ],
        },
        { dependsOn: [dockerGroup], parent }
      );
      const authToken = new oci.identity.AuthToken(
        "hq-auth-token",
        {
          userId: user.id,
          description: "AuthToken for HQ deployment",
        },
        { parent }
      );

      const registryUrl = pulumi.interpolate`ocir.${region}.oci.oraclecloud.com`;

      const hqImage = new docker.Image(
        "hq-image",
        {
          imageName: pulumi.interpolate`${registryUrl}/${repo.namespace}/${repo.displayName}:v1`,
          build: {
            context: "../hq",
            platform: "linux/arm64",
          },
          registry: {
            server: registryUrl,
            username: pulumi.interpolate`${repo.namespace}/${user.name}`,
            password: authToken.token,
          },
        },
        { parent }
      );

      return { hqImage };
    }
  }
}
