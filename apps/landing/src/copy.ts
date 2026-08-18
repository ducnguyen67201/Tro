export type DemoPhase =
  | "idle"
  | "targeting"
  | "listening"
  | "thinking"
  | "solved";

export type Locale = "vi" | "en";

type Feature = {
  number: string;
  title: string;
  body: string;
};

type Step = {
  title: string;
  body: string;
};

export type SiteCopy = {
  meta: {
    title: string;
    description: string;
  };
  language: {
    label: string;
    vietnamese: string;
    english: string;
  };
  header: {
    homeLabel: string;
    navigationLabel: string;
    howItWorks: string;
    whyTro: string;
    backToTop: string;
    systemStatus: string;
    getTro: string;
  };
  hero: {
    practiceWindow: string;
    topic: string;
    explanationWindow: string;
    previewSteps: [string, string, string];
    voiceWindow: string;
    listening: string;
    voicePrompt: string;
    notesFolder: string;
    progressFolder: string;
    codeVariable: string;
    codeValue: string;
    tagline: string;
    description: string;
    primaryCta: string;
    secondaryCta: string;
    shortcutPrefix: string;
    shortcutSuffix: string;
    noteWindow: string;
    noteKicker: string;
    noteBody: string;
    noteAria: string;
  };
  partners: {
    label: string;
    title: string;
    intro: string;
    entries: Array<{
      featuredLabel: string;
      name: string;
      description: string;
      visit: string;
      linkLabel: string;
      website: string;
      logo: string;
    }>;
  };
  demo: {
    label: string;
    title: string;
    statuses: Record<DemoPhase, string>;
    replay: string;
    replayLabel: string;
    address: string;
    progress: string;
    questionNumber: string;
    points: string;
    topic: string;
    question: string;
    answers: [string, string, string];
    listening: string;
    voicePrompt: string;
    thinking: string;
    understood: string;
    solutionEyebrow: string;
    solutionTitle: string;
    steps: [Step, Step, Step];
    answer: string;
    encouragement: string;
  };
  principles: {
    label: string;
    title: string;
    features: [Feature, Feature, Feature];
  };
  download: {
    label: string;
    title: string;
    body: string;
    versionLabel: string;
    version: string;
    platformLabel: string;
    platform: string;
    sizeLabel: string;
    size: string;
    allPlatformsAvailable: string;
    previewPlatformsAvailable: string;
    previewStatus: string;
    unsignedPreviewStatus: string;
    platformsLabel: string;
    platforms: {
      macosApple: {
        badge: string;
        name: string;
        requirements: string;
        size: string;
        status: string;
        cta: string;
        availableStatus?: string;
        availableCta?: string;
      };
      macosIntel: {
        badge: string;
        name: string;
        requirements: string;
        size: string;
        status: string;
        cta: string;
        availableStatus?: string;
        availableCta?: string;
      };
      windows: {
        badge: string;
        name: string;
        requirements: string;
        size: string;
        status: string;
        cta: string;
        availableStatus?: string;
        availableCta?: string;
      };
    };
    accessNote: string;
    previewNote: string;
    unsignedPreviewWarning: string;
    signingDisclosure: string;
    codeSigningPolicy: string;
    privacyPolicy: string;
  };
  closing: {
    codeObject: string;
    firstLine: string;
    secondLine: string;
    cta: string;
  };
  footer: {
    statement: string;
  };
};

export const siteCopy: Record<Locale, SiteCopy> = {
  vi: {
    meta: {
      title: "Tro — Học nhẹ hơn. Tiến xa hơn.",
      description:
        "Tro là người bạn học ưu tiên tiếng Việt: hiểu màn hình, lắng nghe câu hỏi và hướng dẫn bạn từng bước.",
    },
    language: {
      label: "Chọn ngôn ngữ",
      vietnamese: "Tiếng Việt",
      english: "English",
    },
    header: {
      homeLabel: "Trang chủ Tro",
      navigationLabel: "Điều hướng chính",
      howItWorks: "cách hoạt động",
      whyTro: "vì sao chọn tro",
      backToTop: "Về đầu trang",
      systemStatus: "Trạng thái hệ thống Tro",
      getTro: "tải tro",
    },
    hero: {
      practiceWindow: "bài_tập_04",
      topic: "HÀM BẬC HAI",
      explanationWindow: "tro · lời giải thích",
      previewSteps: [
        "nhận ra dạng bài",
        "đối chiếu giá trị",
        "tự tin chọn đáp án",
      ],
      voiceWindow: "giọng nói",
      listening: "Đang lắng nghe",
      voicePrompt: "“Em chưa hiểu chỗ này…”",
      notesFolder: "ghi chú",
      progressFolder: "tiến độ",
      codeVariable: "học",
      codeValue: "dễ",
      tagline: "học nhẹ hơn. tiến xa hơn.",
      description:
        "Một người bạn học ưu tiên tiếng Việt: hiểu ứng dụng bạn đang mở, lắng nghe khi bạn mắc kẹt và cùng bạn đi qua từng bước tiếp theo.",
      primaryCta: "xem tro hoạt động",
      secondaryCta: "vì sao học sinh chọn tro",
      shortcutPrefix: "Nhấn",
      shortcutSuffix: "ở bất cứ đâu · macOS 13+ · Windows 10/11",
      noteWindow: "tro nhắn",
      noteKicker: "Không chỉ là một ô đáp án.",
      noteBody: "Một gia sư có mặt ngay trên màn hình của bạn.",
      noteAria: "Nguyên tắc sản phẩm của Tro",
    },
    partners: {
      label: "Đối tác đồng hành",
      title: "Cùng người học đi xa hơn.",
      intro:
        "Tro hợp tác với những nhà giáo dục tin rằng việc học nên gần gũi, rõ ràng và thực tế.",
      entries: [
        {
          featuredLabel: "Đối tác giáo dục đầu tiên",
          name: "Just Tin English",
          description:
            "Tiếng Anh cho người học lại từ đầu—từ ngữ pháp, từ vựng, nghe và nói đến TOEIC và tiếng Anh thương mại.",
          visit: "Khám phá Just Tin English",
          linkLabel: "Mở trang web Just Tin English trong tab mới",
          website: "https://www.justtinenglish.com/",
          logo: "/partners/just-tin-english.png",
        },
      ],
    },
    demo: {
      label: "Một phím tắt. Một bước tiếp theo thật rõ.",
      title: "Trợ giúp, ngay nơi bạn cần.",
      statuses: {
        idle: "Tro đã sẵn sàng",
        targeting: "Tro thấy chỗ bạn đang vướng",
        listening: "Đang nghe câu hỏi của bạn…",
        thinking: "Đang biến màn hình thành lời giải thích dễ hiểu…",
        solved: "Bạn đã có bước tiếp theo",
      },
      replay: "Xem lại",
      replayLabel: "Xem lại phần minh họa Tro",
      address: "uni.portal / giải-tích / bài-tập-04",
      progress: "4 / 10",
      questionNumber: "Câu 04",
      points: "1 điểm",
      topic: "HÀM BẬC HAI",
      question: "Đồ thị nào khớp với hàm số dưới đây?",
      answers: ["Đỉnh tại (−2, 1)", "Đỉnh tại (2, 1)", "Đỉnh tại (1, 2)"],
      listening: "Đang lắng nghe",
      voicePrompt: "“Em không biết bắt đầu từ đâu…”",
      thinking: "Đang đọc phương trình",
      understood: "Đã hiểu",
      solutionEyebrow: "Cùng làm thật đơn giản",
      solutionTitle: "Đọc tọa độ đỉnh ngay từ phương trình.",
      steps: [
        {
          title: "Nhận ra dạng bài",
          body: "Dạng đỉnh là f(x) = (x − h)² + k.",
        },
        {
          title: "Đối chiếu giá trị",
          body: "Ở đây, h = 2 và k = 1.",
        },
        {
          title: "Tự tin chọn đáp án",
          body: "Đồ thị có đỉnh tại (2, 1).",
        },
      ],
      answer: "Đáp án",
      encouragement: "Giờ hãy tự thử câu tiếp theo nhé.",
    },
    principles: {
      label: "Dành cho khoảnh khắc bạn suýt bỏ cuộc.",
      title: "Từ “em bị kẹt” đến “em làm được rồi”.",
      features: [
        {
          number: "01",
          title: "Hiểu màn hình đang mở",
          body: "Khi tác vụ cần dùng màn hình, Tro quan sát ứng dụng đang hoạt động trước, xin phép trước thay đổi quan trọng và hiển thị viền màu trong lúc điều khiển.",
        },
        {
          number: "02",
          title: "Lắng nghe tự nhiên",
          body: "Hỏi bằng tiếng Việt, tiếng Anh hoặc cả hai—như khi bạn hỏi một người bạn cùng lớp.",
        },
        {
          number: "03",
          title: "Hướng dẫn rồi lùi lại",
          body: "Nhận một lộ trình rõ ràng qua bài toán mà vẫn giữ trọn khoảnh khắc tự mình hiểu ra.",
        },
      ],
    },
    download: {
      label: "Ứng dụng máy tính",
      title: "Tro dành cho Mac và Windows.",
      body: "Chọn đúng phiên bản cho máy của bạn để nhận hướng dẫn bằng giọng nói và ngay trên màn hình khi đang học.",
      versionLabel: "Phiên bản",
      version: "0.1.3",
      platformLabel: "Hệ điều hành",
      platform: "macOS + Windows",
      sizeLabel: "Tình trạng",
      size: "Đang tải thông tin bản phát hành",
      allPlatformsAvailable: "Mac Apple silicon, Mac Intel và Windows có sẵn",
      previewPlatformsAvailable: "Có bản xem trước chưa ký cho máy tính",
      previewStatus: "Bản xem trước",
      unsignedPreviewStatus: "Bản xem trước chưa ký",
      platformsLabel: "Chọn phiên bản Tro cho máy tính",
      platforms: {
        macosApple: {
          badge: "MAC",
          name: "macOS · Apple",
          requirements: "macOS 13+ · Apple silicon",
          size: "139 MB ZIP",
          status: "Sắp ra mắt",
          cta: "Mac sắp ra mắt",
          availableStatus: "Có sẵn",
          availableCta: "Tải Tro cho Mac",
        },
        macosIntel: {
          badge: "MAC",
          name: "macOS · Intel",
          requirements: "macOS 13+ · Intel",
          size: "Bản x64",
          status: "Sắp ra mắt",
          cta: "Mac Intel sắp ra mắt",
          availableStatus: "Có sẵn",
          availableCta: "Tải Tro cho Mac Intel",
        },
        windows: {
          badge: "WIN",
          name: "Windows",
          requirements: "Windows 10/11 · x64",
          size: "Bản x64",
          status: "Sắp ra mắt",
          cta: "Windows sắp ra mắt",
          availableStatus: "Có sẵn",
          availableCta: "Tải Tro cho Windows",
        },
      },
      accessNote: "Đăng nhập bằng Google để bắt đầu sử dụng Tro.",
      previewNote:
        "Các nút tải tự động trỏ tới bản phát hành Tro mới nhất trên GitHub.",
      unsignedPreviewWarning:
        "Các bản Mac và Windows hiện là bản xem trước chưa ký. macOS Gatekeeper hoặc Windows SmartScreen có thể hiển thị cảnh báo trước khi cài đặt.",
      signingDisclosure:
        "Bản Mac ổn định dùng Apple Developer ID và notarization; bản Windows ổn định dùng SignPath.io, chứng thư bởi SignPath Foundation.",
      codeSigningPolicy: "Chính sách ký mã (Code signing policy)",
      privacyPolicy: "Chính sách quyền riêng tư",
    },
    closing: {
      codeObject: "hocSinh.tuTin",
      firstLine: "Bớt mắc kẹt.",
      secondLine: "Giỏi lên mỗi ngày.",
      cta: "Trải nghiệm cách Tro giúp",
    },
    footer: {
      statement: "Ưu tiên tiếng Việt. Luôn đặt người học trước.",
    },
  },
  en: {
    meta: {
      title: "Tro — Study easier. Become your best.",
      description:
        "Tro is the Vietnamese-first desktop tutor that understands the app you have open, listens to your question, and guides the next step.",
    },
    language: {
      label: "Choose language",
      vietnamese: "Tiếng Việt",
      english: "English",
    },
    header: {
      homeLabel: "Tro home",
      navigationLabel: "Main navigation",
      howItWorks: "how it works",
      whyTro: "why tro",
      backToTop: "Back to top",
      systemStatus: "Tro system status",
      getTro: "download",
    },
    hero: {
      practiceWindow: "practice_04",
      topic: "QUADRATIC FUNCTIONS",
      explanationWindow: "tro · explanation",
      previewSteps: [
        "spot the pattern",
        "match the values",
        "choose confidently",
      ],
      voiceWindow: "voice",
      listening: "Listening",
      voicePrompt: "“I don’t understand this part…”",
      notesFolder: "notes",
      progressFolder: "progress",
      codeVariable: "study",
      codeValue: "easy",
      tagline: "study easy. become your best.",
      description:
        "A Vietnamese-first study buddy that understands the app you have open, listens when you are stuck, and walks you through the next step.",
      primaryCta: "see tro work",
      secondaryCta: "why students use tro",
      shortcutPrefix: "Press",
      shortcutSuffix: "anywhere · macOS 13+ · Windows 10/11",
      noteWindow: "tro says",
      noteKicker: "Not another answer box.",
      noteBody: "A tutor that meets you on the screen.",
      noteAria: "Tro product principle",
    },
    partners: {
      label: "Learning partner",
      title: "Better learning, built together.",
      intro:
        "Tro partners with educators who believe learning should feel approachable, clear, and practical.",
      entries: [
        {
          featuredLabel: "Our first education partner",
          name: "Just Tin English",
          description:
            "English for learners starting again—from grammar, vocabulary, listening, and speaking to TOEIC and business English.",
          visit: "Explore Just Tin English",
          linkLabel: "Open the Just Tin English website in a new tab",
          website: "https://www.justtinenglish.com/",
          logo: "/partners/just-tin-english.png",
        },
      ],
    },
    demo: {
      label: "One shortcut. One clear next step.",
      title: "Help, right where you need it.",
      statuses: {
        idle: "Tro is ready",
        targeting: "Tro sees where you are stuck",
        listening: "Listening to your question…",
        thinking: "Turning the screen into a clear explanation…",
        solved: "You have the next step",
      },
      replay: "Replay",
      replayLabel: "Replay the Tro demonstration",
      address: "uni.portal / calculus / practice-04",
      progress: "4 of 10",
      questionNumber: "Question 04",
      points: "1 point",
      topic: "QUADRATIC FUNCTIONS",
      question: "Which graph matches the function below?",
      answers: ["Vertex at (−2, 1)", "Vertex at (2, 1)", "Vertex at (1, 2)"],
      listening: "Listening",
      voicePrompt: "“I don’t know where to start…”",
      thinking: "Reading the equation",
      understood: "Understood",
      solutionEyebrow: "Let’s make it simple",
      solutionTitle: "Read the vertex directly from the equation.",
      steps: [
        {
          title: "Spot the pattern",
          body: "Vertex form is f(x) = (x − h)² + k.",
        },
        {
          title: "Match the values",
          body: "Here, h = 2 and k = 1.",
        },
        {
          title: "Choose with confidence",
          body: "The graph has its vertex at (2, 1).",
        },
      ],
      answer: "Answer",
      encouragement: "Now try the next one yourself.",
    },
    principles: {
      label: "Made for the moment you almost give up.",
      title: "From “I’m stuck” to “I’ve got this.”",
      features: [
        {
          number: "01",
          title: "Understands the open screen",
          body: "When a task needs the screen, Tro observes the active app first, asks before consequential changes, and shows a colored border while it is in control.",
        },
        {
          number: "02",
          title: "Listens naturally",
          body: "Ask in Vietnamese, English, or both—just like you would ask a classmate.",
        },
        {
          number: "03",
          title: "Guides, then steps back",
          body: "Get a clear path through the problem without losing the learning moment.",
        },
      ],
    },
    download: {
      label: "Desktop app",
      title: "Tro for Mac and Windows.",
      body: "Choose the right build for your computer to get voice and on-screen guidance while you study.",
      versionLabel: "Version",
      version: "0.1.3",
      platformLabel: "Systems",
      platform: "macOS + Windows",
      sizeLabel: "Availability",
      size: "Loading release information",
      allPlatformsAvailable:
        "Apple silicon Mac, Intel Mac, and Windows available",
      previewPlatformsAvailable: "Unsigned desktop previews available",
      previewStatus: "Preview",
      unsignedPreviewStatus: "Unsigned preview",
      platformsLabel: "Choose a Tro desktop version",
      platforms: {
        macosApple: {
          badge: "MAC",
          name: "macOS · Apple",
          requirements: "macOS 13+ · Apple silicon",
          size: "139 MB ZIP",
          status: "Coming soon",
          cta: "Mac coming soon",
          availableStatus: "Available",
          availableCta: "Download Tro for Mac",
        },
        macosIntel: {
          badge: "MAC",
          name: "macOS · Intel",
          requirements: "macOS 13+ · Intel",
          size: "x64 build",
          status: "Coming soon",
          cta: "Intel Mac coming soon",
          availableStatus: "Available",
          availableCta: "Download Tro for Intel Mac",
        },
        windows: {
          badge: "WIN",
          name: "Windows",
          requirements: "Windows 10/11 · x64",
          size: "x64 build",
          status: "Coming soon",
          cta: "Windows coming soon",
          availableStatus: "Available",
          availableCta: "Download Tro for Windows",
        },
      },
      accessNote: "Sign in with Google to start using Tro.",
      previewNote:
        "Download buttons automatically follow the latest Tro release on GitHub.",
      unsignedPreviewWarning:
        "The current Mac and Windows builds are unsigned previews. macOS Gatekeeper or Windows SmartScreen may warn before installation.",
      signingDisclosure:
        "Stable Mac builds use Apple Developer ID and notarization; stable Windows builds use SignPath.io, certificate by SignPath Foundation.",
      codeSigningPolicy: "Code signing policy",
      privacyPolicy: "Privacy policy",
    },
    closing: {
      codeObject: "student.confidence",
      firstLine: "Less stuck.",
      secondLine: "More becoming.",
      cta: "Experience the flow",
    },
    footer: {
      statement: "Vietnamese-first. Student-always.",
    },
  },
};
